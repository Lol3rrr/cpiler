use itertools::PeekNth;
use tokenizer::{Operator, Token, TokenData};

use crate::{
    statement::AssignTarget, Expression, ExpressionOperator, ExpressionReason, Statement,
    SyntaxError,
};

use super::starting_type;

#[derive(Debug)]
enum StatementType {
    Assignment,
    Declaration,
    Expression,
}

fn get_stat_type<I>(tokens: &mut PeekNth<I>) -> StatementType
where
    I: Iterator<Item = Token>,
{
    let literal_count = {
        let mut pos = 1;
        let mut count = 1;
        let mut level = 0;

        while let Some(peeked) = tokens.peek_nth(pos) {
            dbg!(&peeked.data);
            match &peeked.data {
                TokenData::Literal { .. } if level == 0 => {
                    count += 1;
                }
                TokenData::Operator(op) => {
                    match op {
                        Operator::Dot => {
                            pos += 1;
                        }
                        Operator::Arrow => {
                            pos += 1;
                        }
                        _ => {}
                    };
                }
                TokenData::Assign(_) => break,
                TokenData::Semicolon => break,
                TokenData::CloseParen if level == 0 => break,
                TokenData::OpenBracket | TokenData::OpenParen => {
                    level += 1;
                }
                TokenData::CloseBracket | TokenData::CloseParen => {
                    level -= 1;
                }
                _ => {}
            };

            pos += 1;
        }

        count
    };

    let contains_assign = {
        let mut contains = false;
        let mut pos = 1;
        let mut level = 0;

        while let Some(peeked) = tokens.peek_nth(pos) {
            match &peeked.data {
                TokenData::Assign(_) => {
                    contains = true;
                    break;
                }
                TokenData::CloseParen if level == 0 => break,
                TokenData::Semicolon => break,
                TokenData::OpenBracket | TokenData::OpenParen => {
                    level += 1;
                }
                TokenData::CloseBracket | TokenData::CloseParen => {
                    level -= 1;
                }
                _ => {}
            };

            pos += 1;
        }

        contains
    };

    match (literal_count, contains_assign) {
        (1, false) => StatementType::Expression,
        (1, true) => StatementType::Assignment,
        (_, _) => StatementType::Declaration,
    }
}

pub fn parse<I>(
    tokens: &mut PeekNth<I>,
    is_termination: &dyn Fn(Token) -> Result<(), SyntaxError>,
) -> Result<Statement, SyntaxError>
where
    I: Iterator<Item = Token>,
{
    let stat_type = get_stat_type(tokens);

    match stat_type {
        StatementType::Assignment => {
            let target = AssignTarget::parse(tokens)?;

            let next_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
            let assign_type = match next_token.data {
                TokenData::Assign(assign_type) => assign_type,
                _ => panic!("Expected '=' but got '{:?}'", next_token),
            };

            let base_exp = Expression::parse(tokens).map_err(|e| match e {
                SyntaxError::UnexpectedEOF => SyntaxError::ExpectedExpression {
                    span: next_token.span,
                    reason: ExpressionReason::Assignment,
                },
                other => other,
            })?;

            let next_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
            is_termination(next_token)?;

            let combine_op = ExpressionOperator::try_from(assign_type);
            let exp = match combine_op {
                Ok(op) => Expression::Operation {
                    left: Box::new(target.to_exp()),
                    operation: op,
                    right: Box::new(base_exp),
                },
                Err(_) => base_exp,
            };

            Ok(Statement::VariableAssignment { target, value: exp })
        }
        StatementType::Declaration => starting_type::parse(tokens, is_termination),
        StatementType::Expression => {
            let exp = Expression::parse(tokens)?;

            let semi_colon_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
            is_termination(semi_colon_token)?;

            Ok(Statement::SingleExpression(exp))
        }
    }
}
