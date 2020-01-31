use crate::ast::*;

pub trait Visitable {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result;
}

impl Visitable for Program {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_program(self)
    }
}

impl Visitable for Declaration {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_declaration(self)
    }
}

impl Visitable for Command {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_command(self)
    }
}

impl Visitable for Expression {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_expression(self)
    }
}

impl Visitable for Condition {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_condition(self)
    }
}

impl Visitable for Value {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_value(self)
    }
}

impl Visitable for Identifier {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_identifier(self)
    }
}

pub trait VisitorResult: Sized {
    fn identity() -> Self;
    fn combine(self, new: Self) -> Self;
    fn combine_collection<I: IntoIterator<Item = Self>>(collection: I) -> Self {
        collection.into_iter().fold(Self::identity(), Self::combine)
    }
}

pub struct ResultCombineErr<T, E: VisitorResult>(Result<T, E>);

impl<T, E: VisitorResult> ResultCombineErr<T, E> {
    pub fn new_err(e: E) -> Self {
        ResultCombineErr(Err(e))
    }

    pub fn new_ok(t: T) -> Self {
        ResultCombineErr(Ok(t))
    }

    pub fn as_result(&self) -> &Result<T, E> {
        &self.0
    }

    pub fn into_result(self) -> Result<T, E> {
        self.into()
    }
}

impl<T, E: VisitorResult> From<Result<T, E>> for ResultCombineErr<T, E> {
    fn from(result: Result<T, E>) -> Self {
        ResultCombineErr(result)
    }
}

impl<T, E: VisitorResult> Into<Result<T, E>> for ResultCombineErr<T, E> {
    fn into(self) -> Result<T, E> {
        self.0
    }
}

pub struct VisitorResultVec<T>(Vec<T>);

impl<T> VisitorResultVec<T> {
    pub fn as_vec(&self) -> &Vec<T> {
        &self.0
    }

    pub fn into_vec(self) -> Vec<T> {
        self.into()
    }
}

impl<T> VisitorResult for VisitorResultVec<T> {
    fn identity() -> Self {
        vec![].into()
    }

    fn combine(mut self, new: Self) -> Self {
        self.0.extend(new.0.into_iter());
        self
    }
}

impl<T> From<Vec<T>> for VisitorResultVec<T> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}

impl<T> From<T> for VisitorResultVec<T> {
    fn from(v: T) -> Self {
        Self(vec![v])
    }
}

impl<T> Into<Vec<T>> for VisitorResultVec<T> {
    fn into(self) -> Vec<T> {
        self.0
    }
}

impl<T: Default, C: VisitorResult> VisitorResult for ResultCombineErr<T, C> {
    fn identity() -> Self {
        ResultCombineErr(Ok(T::default()))
    }

    fn combine(self, new: Self) -> Self {
        if let Err(c) = self.into_result() {
            if let Err(new) = new.into_result() {
                ResultCombineErr::new_err(c.combine(new))
            } else {
                Err(c).into()
            }
        } else {
            new
        }
    }
}

impl VisitorResult for () {
    fn identity() -> Self {
        ()
    }

    fn combine(self, _: Self) -> Self {
        ()
    }

    fn combine_collection<I: IntoIterator<Item = Self>>(_: I) -> Self {
        ()
    }
}

pub trait Visitor: Sized {
    type Result: VisitorResult;

    fn visit<V: Visitable>(&mut self, visitable: &V) -> Self::Result {
        visitable.accept(self)
    }

    fn visit_collection<'a, V: Visitable + 'a, I: IntoIterator<Item = &'a V>>(
        &mut self,
        collection: I,
    ) -> Self::Result
    where
        I::IntoIter: ExactSizeIterator,
    {
        let iter = collection.into_iter();
        let mut results = Vec::with_capacity(iter.len());
        for v in iter {
            results.push(self.visit(v));
        }
        Self::Result::combine_collection(results)
    }

    fn visit_program(&mut self, program: &Program) -> Self::Result {
        let res = if let Some(declarations) = &program.declarations {
            self.visit_declarations(declarations)
        } else {
            Self::Result::identity()
        };

        res.combine(self.visit_commands(&program.commands))
    }

    fn visit_declarations(&mut self, declarations: &Declarations) -> Self::Result {
        self.visit_collection(declarations)
    }

    fn visit_declaration(&mut self, declaration: &Declaration) -> Self::Result;

    fn visit_if_else_command(
        &mut self,
        condition: &Condition,
        positive: &Commands,
        negative: &Commands,
    ) -> Self::Result {
        self.visit(condition)
            .combine(self.visit_commands(positive))
            .combine(self.visit_commands(negative))
    }

    fn visit_if_command(&mut self, condition: &Condition, positive: &Commands) -> Self::Result {
        self.visit(condition).combine(self.visit_commands(positive))
    }

    fn visit_while_command(&mut self, condition: &Condition, commands: &Commands) -> Self::Result {
        self.visit(condition).combine(self.visit_commands(commands))
    }

    fn visit_do_command(&mut self, commands: &Commands, condition: &Condition) -> Self::Result {
        self.visit_commands(commands).combine(self.visit(condition))
    }

    fn visit_for_command(
        &mut self,
        _counter: &str,
        _ascending: bool,
        from: &Value,
        to: &Value,
        commands: &Commands,
    ) -> Self::Result {
        self.visit(from)
            .combine(self.visit(to))
            .combine(self.visit_commands(commands))
    }

    fn visit_read_command(&mut self, target: &Identifier) -> Self::Result {
        self.visit(target)
    }

    fn visit_write_command(&mut self, value: &Value) -> Self::Result {
        self.visit(value)
    }

    fn visit_assign_command(&mut self, target: &Identifier, expr: &Expression) -> Self::Result {
        self.visit(target).combine(self.visit(expr))
    }

    fn visit_commands(&mut self, commands: &Commands) -> Self::Result {
        self.visit_collection(commands)
    }

    fn visit_command(&mut self, command: &Command) -> Self::Result {
        match command {
            Command::IfElse {
                condition,
                positive,
                negative,
            } => self.visit_if_else_command(condition, positive, negative),
            Command::If {
                condition,
                positive,
            } => self.visit_if_command(condition, positive),
            Command::While {
                condition,
                commands,
            } => self.visit_while_command(condition, commands),
            Command::Do {
                commands,
                condition,
            } => self.visit_do_command(commands, condition),
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

    fn visit_simple_expression(&mut self, value: &Value) -> Self::Result {
        self.visit(value)
    }

    fn visit_compound_expression(
        &mut self,
        left: &Value,
        _op: &ExprOp,
        right: &Value,
    ) -> Self::Result {
        self.visit(left).combine(self.visit(right))
    }

    fn visit_expression(&mut self, expr: &Expression) -> Self::Result {
        match expr {
            Expression::Simple { value } => self.visit_simple_expression(value),
            Expression::Compound { left, op, right } => {
                self.visit_compound_expression(left, op, right)
            }
        }
    }

    fn visit_condition(&mut self, condition: &Condition) -> Self::Result {
        self.visit(&condition.left)
            .combine(self.visit(&condition.right))
    }

    fn visit_num_value(&mut self, num: i64) -> Self::Result;
    fn visit_identifier_value(&mut self, identifier: &Identifier) -> Self::Result {
        self.visit(identifier)
    }

    fn visit_value(&mut self, value: &Value) -> Self::Result {
        match value {
            Value::Num(num) => self.visit_num_value(*num),
            Value::Identifier(identifier) => self.visit_identifier_value(identifier),
        }
    }

    fn visit_identifier(&mut self, identifier: &Identifier) -> Self::Result;
}
