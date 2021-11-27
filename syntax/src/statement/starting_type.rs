use general::{Span, SpanData};
use itertools::PeekNth;
use tokenizer::{Assignment, Token, TokenData};

use super::structs;
use crate::{
    ExpectedToken, Expression, ExpressionReason, FunctionArgument, Identifier, Scope, Statement,
    SyntaxError, TypeToken,
};

/// This gets called if we want to parse a new Statement and notice that it started with a
/// Type, meaning it can only be either a variable or function declaration/definition
pub fn parse<I>(
    tokens: &mut PeekNth<I>,
    is_termination: &dyn Fn(Token) -> Result<(), SyntaxError>,
) -> Result<Statement, SyntaxError>
where
    I: Iterator<Item = Token>,
{
    let ty_tokens = TypeToken::parse(tokens)?;
    let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;
    let ty_tokens = match (ty_tokens, &peeked.data) {
        (TypeToken::StructType { name }, TokenData::OpenBrace) => {
            let members = structs::StructMembers::parse(tokens)?;

            return Ok(Statement::StructDefinition { name, members });
        }
        (t, _) => t,
    };

    let name = Identifier::parse(tokens)?;

    let array_peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;
    let f_type = match &array_peeked.data {
        TokenData::OpenBracket => {
            let _ = tokens.next();

            let tmp_peeked = tokens.peek();
            match tmp_peeked {
                Some(tok) if &tok.data == &TokenData::CloseBracket => {
                    let _ = tokens.next();

                    TypeToken::ArrayType {
                        base: Box::new(ty_tokens),
                        size: None,
                    }
                }
                _ => {
                    let size_exp = Expression::parse(tokens)?;

                    let close_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                    match close_token.data {
                        TokenData::CloseBracket => {}
                        other => panic!("Expected ']' but got '{:?}'", other),
                    };

                    TypeToken::ArrayType {
                        base: Box::new(ty_tokens),
                        size: Some(Box::new(size_exp)),
                    }
                }
            }
        }
        _ => ty_tokens,
    };

    let peeked = tokens.peek().unwrap();

    match &peeked.data {
        TokenData::OpenParen => {
            let _ = tokens.next();

            let mut arguments: Vec<SpanData<FunctionArgument>> = Vec::new();
            while let Some(tmp_tok) = tokens.peek() {
                // TODO
                dbg!(&tmp_tok);
                match &tmp_tok.data {
                    TokenData::CloseParen => {
                        let _ = tokens.next();
                        break;
                    }
                    _ => {}
                };

                let source = tmp_tok.span.source().clone();
                let start = tmp_tok.span.source_area().start;

                let ty = TypeToken::parse(tokens)?;

                let name_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                let name = match name_token.data {
                    TokenData::Literal { content } => Identifier(SpanData {
                        span: name_token.span,
                        data: content,
                    }),
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::Identifier]),
                            got: name_token.span,
                        })
                    }
                };

                let arg_span = Span::new_arc_source(source, start..name.0.span.source_area().end);

                arguments.push(SpanData {
                    span: arg_span,
                    data: FunctionArgument { name, ty },
                });

                let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;
                match &peeked.data {
                    TokenData::Comma => {
                        let _ = tokens.next();
                    }
                    _ => {}
                };
            }

            let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
            match &next_tok.data {
                TokenData::OpenBrace => {
                    let inner_scope = Scope::parse(tokens)?;

                    Ok(Statement::FunctionDefinition {
                        name,
                        r_type: f_type,
                        arguments,
                        body: inner_scope,
                    })
                }
                TokenData::Semicolon => Ok(Statement::FunctionDeclaration {
                    name,
                    r_type: f_type,
                    arguments,
                }),
                other => panic!("Expected a {{ or ; but got: {:?}", other),
            }
        }
        TokenData::Assign(assign_type) => {
            match assign_type {
                Assignment::Assign => {}
                other => {
                    panic!("Expected a normal Assignment('=') but got '{}'", other)
                }
            };

            let tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;

            let exp = Expression::parse(tokens).map_err(|e| match e {
                SyntaxError::UnexpectedEOF => SyntaxError::ExpectedExpression {
                    span: tok.span,
                    reason: ExpressionReason::Assignment,
                },
                other => other,
            })?;

            let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
            is_termination(next_tok)?;

            Ok(Statement::VariableDeclarationAssignment {
                ty: f_type,
                name,
                value: exp,
            })
        }
        _ if is_termination(peeked.clone()).is_ok() => {
            let _ = tokens.next();
            Ok(Statement::VariableDeclaration { name, ty: f_type })
        }
        tok_data => {
            panic!("Unexpected Token: {:?}", tok_data);
        }
    }
}
