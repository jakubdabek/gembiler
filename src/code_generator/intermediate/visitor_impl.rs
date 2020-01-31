use super::CodeGenerator;
use crate::code_generator::intermediate::variable::Variable;
use crate::code_generator::intermediate::{Access, Constant, Instruction};
use parser::ast;
use parser::ast::visitor::Visitor;
use parser::ast::{ExprOp, RelOp};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum Order {
    First,
    Second,
}

impl CodeGenerator {
    fn emit_if_else<F: FnMut(&mut Self, Order)>(
        &mut self,
        condition: &ast::Condition,
        mut emit_body: F,
    ) {
        let negative_label = self.new_label();
        let endif_label = self.new_label();

        self.visit(condition);

        let (first_order, second_order) = match condition.op {
            RelOp::NEQ | RelOp::LEQ | RelOp::GEQ => (Order::First, Order::Second),
            RelOp::EQ | RelOp::LT | RelOp::GT => (Order::Second, Order::First),
        };

        let cond_jump = match condition.op {
            RelOp::EQ | RelOp::NEQ => Instruction::JZero {
                label: negative_label,
            },
            RelOp::GT | RelOp::LEQ => Instruction::JPositive {
                label: negative_label,
            },
            RelOp::LT | RelOp::GEQ => Instruction::JNegative {
                label: negative_label,
            },
        };

        self.emit(cond_jump);
        emit_body(self, first_order);
        self.emit(Instruction::Jump { label: endif_label });
        self.emit(Instruction::Label {
            label: negative_label,
        });
        emit_body(self, second_order);
        self.emit(Instruction::Label { label: endif_label });
    }

    fn emit_do<F: FnMut(&mut Self)>(&mut self, condition: &ast::Condition, mut emit_body: F) {
        // do { commands } while(condition)
        // is the same as:
        // start: { commands } if(condition) jump start;
        let start_label = self.new_label();

        self.emit(Instruction::Label { label: start_label });
        emit_body(self);
        let emit_jump = |gen: &mut Self, order| {
            if order == Order::First {
                gen.emit(Instruction::Jump { label: start_label });
            }
        };
        self.emit_if_else(condition, emit_jump);
    }

    fn emit_while<F: FnMut(&mut Self)>(&mut self, condition: &ast::Condition, mut emit_body: F) {
        // while(condition) { commands }
        // is the same as:
        // if(condition) { do { commands } while(condition) }
        let emit_if = |gen: &mut Self, order| {
            if order == Order::First {
                gen.emit_do(condition, &mut emit_body);
            }
        };
        self.emit_if_else(condition, emit_if);
    }
}

impl Visitor for CodeGenerator {
    type Result = ();

    fn visit_declaration(&mut self, declaration: &ast::Declaration) -> Self::Result {
        let var = match declaration {
            ast::Declaration::Var { name } => Variable::Unit { name: name.clone() },
            ast::Declaration::Array { name, start, end } => Variable::Array {
                name: name.clone(),
                start: *start,
                end: *end,
            },
        };

        self.add_global(var);
    }

    fn visit_if_else_command(
        &mut self,
        condition: &ast::Condition,
        positive: &ast::Commands,
        negative: &ast::Commands,
    ) -> Self::Result {
        let emit = |gen: &mut Self, order| match order {
            Order::First => gen.visit_commands(positive),
            Order::Second => gen.visit_commands(negative),
        };
        self.emit_if_else(condition, emit);
    }

    fn visit_if_command(
        &mut self,
        condition: &ast::Condition,
        positive: &ast::Commands,
    ) -> Self::Result {
        let emit = |gen: &mut Self, order| {
            if order == Order::First {
                gen.visit_commands(positive);
            }
        };
        self.emit_if_else(condition, emit);
    }

    fn visit_while_command(
        &mut self,
        condition: &ast::Condition,
        commands: &ast::Commands,
    ) -> Self::Result {
        self.emit_while(condition, |gen: &mut Self| gen.visit_commands(commands));
    }

    fn visit_do_command(
        &mut self,
        commands: &ast::Commands,
        condition: &ast::Condition,
    ) -> Self::Result {
        self.emit_do(condition, |gen: &mut Self| gen.visit_commands(commands));
    }

    fn visit_for_command(
        &mut self,
        counter: &str,
        ascending: bool,
        from: &ast::Value,
        to: &ast::Value,
        commands: &ast::Commands,
    ) -> Self::Result {
        let counter_var = self.add_local(Variable::Unit {
            name: counter.to_owned(),
        });
        let tmp = self.add_local(Variable::Unit {
            name: counter.to_owned() + "$to",
        });

        self.emit(Instruction::PreStore {
            access: Access::Variable(counter_var),
        });
        self.visit(from);
        self.emit_load_visited();
        self.emit(Instruction::Store {
            access: Access::Variable(counter_var),
        });

        self.emit(Instruction::PreStore {
            access: Access::Variable(tmp),
        });
        self.visit(to);
        self.emit_load_visited();
        self.emit(Instruction::Store {
            access: Access::Variable(tmp),
        });

        let counter_name = self
            .context
            .get_variable(&counter_var)
            .variable()
            .name()
            .to_owned();
        debug_assert_eq!(counter_name.as_str(), counter);
        let tmp_name = self.context.get_variable(&tmp).variable().name().to_owned();
        debug_assert_eq!(tmp_name.as_str(), (counter_name.clone() + "$to").as_str());

        self.emit_while(
            &ast::Condition {
                left: ast::Value::Identifier(ast::Identifier::VarAccess {
                    name: counter_name.clone(),
                }),
                op: if ascending {
                    ast::RelOp::LEQ
                } else {
                    ast::RelOp::GEQ
                },
                right: ast::Value::Identifier(ast::Identifier::VarAccess {
                    name: tmp_name.clone(),
                }),
            },
            |gen| {
                gen.visit_commands(commands);
                gen.visit_assign_command(
                    &ast::Identifier::VarAccess {
                        name: counter_name.clone(),
                    },
                    &ast::Expression::Compound {
                        left: ast::Value::Identifier(ast::Identifier::VarAccess {
                            name: counter_name.clone(),
                        }),
                        op: if ascending {
                            ast::ExprOp::Plus
                        } else {
                            ast::ExprOp::Minus
                        },
                        right: ast::Value::Num(1),
                    },
                );
            },
        );

        self.pop_local(tmp);
        self.pop_local(counter_var);
    }

    fn visit_read_command(&mut self, target: &ast::Identifier) -> Self::Result {
        self.visit(target);
        self.emit_pre_store_visited();
        self.emit(Instruction::Get);
        self.emit_store_visited();
    }

    fn visit_write_command(&mut self, value: &ast::Value) -> Self::Result {
        self.visit(value);
        self.emit_load_visited();
        self.emit(Instruction::Put);
    }

    fn visit_assign_command(
        &mut self,
        target: &ast::Identifier,
        expr: &ast::Expression,
    ) -> Self::Result {
        self.visit(target);
        self.emit_pre_store_visited();
        self.visit(expr);
        self.emit_store_visited();
    }

    //
    // fn visit_commands(&mut self, commands: &ast::Commands) -> Self::Result {
    //     unimplemented!()
    // }
    //
    // fn visit_command(&mut self, command: &ast::Command) -> Self::Result {
    //     unimplemented!()
    // }

    fn visit_simple_expression(&mut self, value: &ast::Value) -> Self::Result {
        self.visit(value);
        self.emit_load_visited();
    }

    fn visit_compound_expression(
        &mut self,
        left: &ast::Value,
        op: &ast::ExprOp,
        right: &ast::Value,
    ) -> Self::Result {
        self.visit(left);
        let left = self.pop_access();
        self.visit(right);
        let right = self.pop_access();
        self.emit(Instruction::Operation {
            left,
            op: (*op).into(),
            right,
        });
    }

    // fn visit_expression(&mut self, expr: &ast::Expression) -> Self::Result {
    //     unimplemented!()
    // }

    fn visit_condition(&mut self, condition: &ast::Condition) -> Self::Result {
        self.visit_compound_expression(&condition.left, &ExprOp::Minus, &condition.right);
    }

    fn visit_num_value(&mut self, num: i64) -> Self::Result {
        self.context.register_constant(Constant(num));
        self.push_access(Access::Constant(Constant(num)));
    }

    fn visit_identifier(&mut self, identifier: &ast::Identifier) -> Self::Result {
        use ast::Identifier::*;
        match identifier {
            ArrAccess { name, index } => {
                let name_index = self.find_variable_by_name(name).unwrap().id();
                let index_index = self.find_variable_by_name(index).unwrap().id();

                self.push_access(Access::ArrayDynamic(name_index, index_index))
            }
            ArrConstAccess { name, index } => {
                let name_index = self.find_variable_by_name(name).unwrap().id();
                self.context.register_constant(Constant(*index));
                self.push_access(Access::ArrayStatic(name_index, Constant(*index)));
            }
            VarAccess { name } => {
                let name_index = self.find_variable_by_name(name).unwrap().id();
                self.push_access(Access::Variable(name_index));
            }
        }
    }
}
