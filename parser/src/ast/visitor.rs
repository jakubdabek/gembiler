use crate::ast::*;

pub trait Visitable {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), V::Err>;
}

impl Visitable for Program {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), V::Err> {
        visitor.visit_program(self)
    }
}

impl Visitable for Declaration {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), V::Err> {
        visitor.visit_declaration(self)
    }
}

impl Visitable for Command {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), V::Err> {
        visitor.visit_command(self)
    }
}

impl Visitable for Expression {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), V::Err> {
        visitor.visit_expression(self)
    }
}

impl Visitable for Condition {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), V::Err> {
        visitor.visit_condition(self)
    }
}

impl Visitable for Value {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), V::Err> {
        visitor.visit_value(self)
    }
}

impl Visitable for Identifier {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), V::Err> {
        visitor.visit_identifier(self)
    }
}

pub trait Visitor: Sized {
    type Err;

    fn visit<V: Visitable>(&mut self, visitable: &V) -> Result<(), Self::Err> {
        visitable.accept(self)
    }


    fn visit_program(&mut self, program: &Program) -> Result<(), Self::Err> {
        if let Some(declarations) = &program.declarations {
            self.visit_declarations(declarations)?
        }

        self.visit_commands(&program.commands)
    }

    fn visit_declarations(&mut self, declarations: &Declarations) -> Result<(), Self::Err> {
        for declaration in declarations {
            self.visit(declaration)?;
        }

        Ok(())
    }

    fn visit_declaration(&mut self, declaration: &Declaration) -> Result<(), Self::Err>;

    fn visit_if_else_command(&mut self, condition: &Condition, positive: &Commands, negative: &Commands) -> Result<(), Self::Err> {
        self.visit(condition)?;
        self.visit_commands(positive)?;
        self.visit_commands(negative)
    }

    fn visit_if_command(&mut self, condition: &Condition, positive: &Commands) -> Result<(), Self::Err> {
        self.visit(condition)?;
        self.visit_commands(positive)
    }

    fn visit_while_command(&mut self, condition: &Condition, commands: &Commands) -> Result<(), Self::Err> {
        self.visit(condition)?;
        self.visit_commands(commands)
    }

    fn visit_do_command(&mut self, commands: &Commands, condition: &Condition) -> Result<(), Self::Err> {
        self.visit_commands(commands)?;
        self.visit(condition)
    }

    fn visit_for_command(&mut self, counter: &str, ascending: bool, from: &Value, to: &Value, commands: &Commands) -> Result<(), Self::Err> {
        self.visit(from)?;
        self.visit(to)?;
        self.visit_commands(commands)
    }

    fn visit_read_command(&mut self, target: &Identifier) -> Result<(), Self::Err> {
        self.visit(target)
    }

    fn visit_write_command(&mut self, value: &Value) -> Result<(), Self::Err> {
        self.visit(value)
    }

    fn visit_assign_command(&mut self, target: &Identifier, expr: &Expression) -> Result<(), Self::Err> {
        self.visit(target)?;
        self.visit(expr)
    }

    fn visit_commands(&mut self, commands: &Commands) -> Result<(), Self::Err> {
        for command in commands {
            self.visit(command)?;
        }

        Ok(())
    }

    fn visit_command(&mut self, command: &Command) -> Result<(), Self::Err> {
        match command {
            Command::IfElse {
                condition,
                positive,
                negative
            } => self.visit_if_else_command(condition, positive, negative),
            Command::If { condition, positive } => self.visit_if_command(condition, positive),
            Command::While { condition, commands } => self.visit_while_command(condition, commands),
            Command::Do { commands, condition } => self.visit_do_command(commands, condition),
            Command::For {
                counter,
                ascending,
                from,
                to,
                commands,
            } => self.visit_for_command(counter, *ascending, from, to, commands),
            Command::Read { target } => self.visit_read_command(target),
            Command::Write { value } => self.visit_write_command(value),
            Command::Assign { target, expr } => self.visit_assign_command(target, expr),
        }
    }

    fn visit_simple_expression(&mut self, value: &Value) -> Result<(), Self::Err> {
        self.visit(value)
    }

    fn visit_compound_expression(&mut self, left: &Value, op: &ExprOp, right: &Value) -> Result<(), Self::Err> {
        self.visit(left)?;
        self.visit(right)
    }

    fn visit_expression(&mut self, expr: &Expression) -> Result<(), Self::Err> {
        match expr {
            Expression::Simple { value } => self.visit_simple_expression(value),
            Expression::Compound {
                left,
                op,
                right,
            } => self.visit_compound_expression(left, op, right),
        }
    }

    fn visit_condition(&mut self, condition: &Condition) -> Result<(), Self::Err> {
        self.visit(&condition.left)?;
        self.visit(&condition.right)
    }

    fn visit_num_value(&mut self, num: i64) -> Result<(), Self::Err>;
    fn visit_identifier_value(&mut self, identifier: &Identifier) -> Result<(), Self::Err> {
        self.visit(identifier)
    }

    fn visit_value(&mut self, value: &Value) -> Result<(), Self::Err> {
        match value {
            Value::Num(num) => self.visit_num_value(*num),
            Value::Identifier(identifier) => self.visit_identifier_value(identifier),
        }
    }

    fn visit_identifier(&mut self, command: &Identifier) -> Result<(), Self::Err>;
}
