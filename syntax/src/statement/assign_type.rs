use tokenizer::Assignment;

use crate::{Expression, ExpressionOperator};

pub fn convert_assign<F>(value: Expression, ty: Assignment, left: F) -> Expression
where
    F: FnOnce() -> Expression,
{
    match ty {
        Assignment::Assign => value,
        Assignment::Add => Expression::Operation {
            left: Box::new(left()),
            right: Box::new(value),
            operation: ExpressionOperator::Add,
        },
        _ => todo!(""),
    }
}
