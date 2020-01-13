#![allow(dead_code)]

use parser::ast::visitor::Visitor;
use parser::ast;
use super::CodeGenerator;
use crate::code_generator::intermediate::variable::Variable;
use crate::code_generator::intermediate::{Instruction, Access, Constant, Label};
use parser::ast::{ExprOp, RelOp};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum Order {
    First,
    Second,
}

impl CodeGenerator {
    fn emit_if_else<F: Fn(&mut Self, Order)>(&mut self, condition: &ast::Condition, emit: F) {
        let negative_label = self.new_label();
        let endif_label = self.new_label();

        self.visit(condition);

        let (first_order, second_order) = match condition.op {
            RelOp::EQ
            | RelOp::LEQ
            | RelOp::GEQ => (Order::Second, Order::First),
            RelOp::NEQ
            | RelOp::LT
            | RelOp::GT => (Order::First, Order::Second),
        };

        let cond_jump = match condition.op {
            RelOp::EQ
            | RelOp::NEQ => Instruction::JZero { label: negative_label },
            RelOp::GT
            | RelOp::LEQ => Instruction::JPositive { label: negative_label },
            RelOp::LT
            | RelOp::GEQ => Instruction::JNegative { label: negative_label },
        };

        self.emit(cond_jump);
        emit(self, first_order);
        self.emit(Instruction::Jump { label: endif_label });
        self.emit(Instruction::Label { label: negative_label });
        emit(self, second_order);
        self.emit(Instruction::Label { label: endif_label });
    }
}

impl Visitor for CodeGenerator {
    type Result = ();

    fn visit_declaration(&mut self, declaration: &ast::Declaration) -> Self::Result {
        let var = match declaration {
            ast::Declaration::Var { name } => Variable::Unit { name: name.clone() },
            ast::Declaration::Array {
                name,
                start,
                end
            } => Variable::Array { name: name.clone(), start: *start, end: *end },
        };

        self.add_global(var);
    }

    fn visit_if_else_command(&mut self, condition: &ast::Condition, positive: &ast::Commands, negative: &ast::Commands) -> Self::Result {
        let emit = |gen: &mut Self, order| {
            match order {
                Order::First => gen.visit_commands(positive),
                Order::Second => gen.visit_commands(negative),
            }
        };
        self.emit_if_else(condition, emit);
    }

    fn visit_if_command(&mut self, condition: &ast::Condition, positive: &ast::Commands) -> Self::Result {
        let emit = |gen: &mut Self, order| {
            if order == Order::First {
                gen.visit_commands(positive);
            }
        };
        self.emit_if_else(condition, emit);
    }

    fn visit_while_command(&mut self, condition: &ast::Condition, commands: &ast::Commands) -> Self::Result {
        // while(condition) { commands }
        // is the same as:
        // if(condition) { do { commands } while(condition) }
        let emit_if = |gen: &mut Self, order| {
            if order == Order::First {
                gen.visit_do_command(commands, condition);
            }
        };
        self.emit_if_else(condition, emit_if);
    }

    fn visit_do_command(&mut self, commands: &ast::Commands, condition: &ast::Condition) -> Self::Result {
        // do { commands } while(condition)
        // is the same as:
        // start: { commands } if(condition) jump start;
        let start_label = self.new_label();

        self.emit(Instruction::Label { label: start_label });
        self.visit_commands(commands);
        let emit = |gen: &mut Self, order| {
            if order == Order::First {
                gen.emit(Instruction::Jump { label: start_label });
            }
        };
        self.emit_if_else(condition, emit);
    }

    fn visit_for_command(&mut self, _counter: &str, _ascending: bool, from: &ast::Value, to: &ast::Value, commands: &ast::Commands) -> Self::Result {
        unimplemented!()
    }

    fn visit_read_command(&mut self, target: &ast::Identifier) -> Self::Result {
        self.visit(target);
        self.emit_pre_store();
        self.emit(Instruction::Get);
        self.emit_store();
    }

    fn visit_write_command(&mut self, value: &ast::Value) -> Self::Result {
        self.visit(value);
        self.emit_load();
        self.emit(Instruction::Put);
    }

    fn visit_assign_command(&mut self, target: &ast::Identifier, expr: &ast::Expression) -> Self::Result {
        self.visit(target);
        self.emit_pre_store();
        self.visit(expr);
        self.emit_store();
    }
//
//    fn visit_commands(&mut self, commands: &ast::Commands) -> Self::Result {
//        unimplemented!()
//    }
//
//    fn visit_command(&mut self, command: &ast::Command) -> Self::Result {
//        unimplemented!()
//    }

    fn visit_simple_expression(&mut self, value: &ast::Value) -> Self::Result {
        self.visit(value);
        self.emit_load();
    }

    fn visit_compound_expression(&mut self, left: &ast::Value, op: &ast::ExprOp, right: &ast::Value) -> Self::Result {
        self.visit(right);
        self.emit_load();
        let temp = self.emit_temporary_store();
        self.visit(left);
        self.emit_load();
        self.emit(Instruction::Operation { op: *op, operand: temp });
    }

//    fn visit_expression(&mut self, expr: &ast::Expression) -> Self::Result {
//        unimplemented!()
//    }

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
            },
            ArrConstAccess { name, index } => {
                let name_index = self.find_variable_by_name(name).unwrap().id();
                self.context.register_constant(Constant(*index));
                self.push_access(Access::ArrayStatic(name_index, Constant(*index)));
            },
            VarAccess { name } => {
                let name_index = self.find_variable_by_name(name).unwrap().id();
                self.push_access(Access::Variable(name_index));
            }
        }
    }
}