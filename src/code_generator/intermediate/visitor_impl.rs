#![allow(dead_code)]

use parser::ast::visitor::Visitor;
use parser::ast;
use super::CodeGenerator;
use crate::code_generator::intermediate::variable::Variable;
use crate::code_generator::intermediate::{Instruction, Access, Constant};

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
        unimplemented!()
    }

    fn visit_if_command(&mut self, condition: &ast::Condition, positive: &ast::Commands) -> Self::Result {
        unimplemented!()
    }

    fn visit_while_command(&mut self, condition: &ast::Condition, commands: &ast::Commands) -> Self::Result {
        unimplemented!()
    }

    fn visit_do_command(&mut self, commands: &ast::Commands, condition: &ast::Condition) -> Self::Result {
        unimplemented!()
    }

    fn visit_for_command(&mut self, _counter: &str, _ascending: bool, from: &ast::Value, to: &ast::Value, commands: &ast::Commands) -> Self::Result {
        unimplemented!()
    }

    fn visit_read_command(&mut self, target: &ast::Identifier) -> Self::Result {
        unimplemented!()
    }

    fn visit_write_command(&mut self, value: &ast::Value) -> Self::Result {
        self.visit(value);
        self.emit(Instruction::Write);
    }

    fn visit_assign_command(&mut self, target: &ast::Identifier, expr: &ast::Expression) -> Self::Result {
        unimplemented!()
    }

    fn visit_commands(&mut self, commands: &ast::Commands) -> Self::Result {
        unimplemented!()
    }

    fn visit_command(&mut self, command: &ast::Command) -> Self::Result {
        unimplemented!()
    }

    fn visit_simple_expression(&mut self, value: &ast::Value) -> Self::Result {
        unimplemented!()
    }

    fn visit_compound_expression(&mut self, left: &ast::Value, _op: &ast::ExprOp, right: &ast::Value) -> Self::Result {
        unimplemented!()
    }

    fn visit_expression(&mut self, expr: &ast::Expression) -> Self::Result {
        unimplemented!()
    }

    fn visit_condition(&mut self, condition: &ast::Condition) -> Self::Result {
        unimplemented!()
    }

    fn visit_num_value(&mut self, num: i64) -> Self::Result {
        self.accesses().access_constant(num);
    }

    fn visit_identifier(&mut self, identifier: &ast::Identifier) -> Self::Result {
        use ast::Identifier::*;
        match identifier {
            ArrAccess { name, index } => {
                let index_index = self.find_variable_by_name(index).unwrap().id();
                let name_index = self.find_variable_by_name(name).unwrap().id();
                let accesses = self.accesses();
                accesses.access_variable(index_index);
                accesses.access_variable(name_index);
                accesses.access_array();
            },
            ArrConstAccess { name, index } => {
                let name_index = self.find_variable_by_name(name).unwrap().id();
                let accesses = self.accesses();
                accesses.access_constant(*index);
                accesses.access_variable(name_index);
                accesses.access_array();
            },
            VarAccess { name } => {
                let name_index = self.find_variable_by_name(name).unwrap().id();
                let accesses = self.accesses();
                accesses.access_variable(name_index);
            }
        }
    }
}