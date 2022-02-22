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
        Assignment::Sub => Expression::Operation {
            left: Box::new(left()),
            right: Box::new(value),
            operation: ExpressionOperator::Sub,
        },
        Assignment::Multiply => Expression::Operation {
            left: Box::new(left()),
            right: Box::new(value),
            operation: ExpressionOperator::Multiply,
        },
        Assignment::Divide => Expression::Operation {
            left: Box::new(left()),
            right: Box::new(value),
            operation: ExpressionOperator::Divide,
        },
        Assignment::Modulo => Expression::Operation {
            left: Box::new(left()),
            right: Box::new(value),
            operation: ExpressionOperator::Modulo,
        },
        Assignment::BitwiseAnd => Expression::Operation {
            left: Box::new(left()),
            right: Box::new(value),
            operation: ExpressionOperator::BitwiseAnd,
        },
        Assignment::BitwiseOr => Expression::Operation {
            left: Box::new(left()),
            right: Box::new(value),
            operation: ExpressionOperator::BitwiseOr,
        },
        Assignment::BitwiseXor => Expression::Operation {
            left: Box::new(left()),
            right: Box::new(value),
            operation: ExpressionOperator::BitwiseXor,
        },
        Assignment::ShiftRight => Expression::Operation {
            left: Box::new(left()),
            right: Box::new(value),
            operation: ExpressionOperator::ShiftRight,
        },
        Assignment::ShiftLeft => Expression::Operation {
            left: Box::new(left()),
            right: Box::new(value),
            operation: ExpressionOperator::ShiftLeft,
        },
        other => todo!("Convert Assignment: {:?}", other),
    }
}
