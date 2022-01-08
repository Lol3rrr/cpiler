use general::{Span, SpanData};
use itertools::PeekNth;
use tokenizer::{Assignment, Token, TokenData};

use super::structs;
use crate::{
    statement::{enums, FunctionHead},
    EOFContext, ExpectedToken, Expression, ExpressionReason, FunctionArgument, Identifier, Scope,
    Statement, SyntaxError, TypeToken,
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
    let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
        ctx: EOFContext::Statement,
    })?;
    let ty_tokens = match (ty_tokens, &peeked.data) {
        (TypeToken::StructType { name }, TokenData::OpenBrace) => {
            let start_span = peeked.span.clone();

            let members = structs::StructMembers::parse(tokens)?;

            let end_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                ctx: EOFContext::Statement,
            })?;
            match end_tok.data {
                TokenData::Semicolon => {}
                _ => {
                    return Err(SyntaxError::UnexpectedToken {
                        got: end_tok.span,
                        expected: Some(vec![ExpectedToken::Semicolon]),
                    });
                }
            };
            let end_span = end_tok.span;
            let entire_span = Span::new_arc_source(
                start_span.source().clone(),
                start_span.source_area().start..end_span.source_area().end,
            );

            return Ok(Statement::StructDefinition {
                name,
                members,
                definition: entire_span,
            });
        }
        (TypeToken::EnumType { name }, TokenData::OpenBrace) => {
            dbg!(&name);

            let variants = enums::EnumVariants::parse(tokens)?;
            dbg!(&variants);

            let end_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                ctx: EOFContext::Statement,
            })?;
            match end_token.data {
                TokenData::Semicolon => {}
                _ => {
                    return Err(SyntaxError::UnexpectedToken {
                        got: end_token.span,
                        expected: Some(vec![ExpectedToken::Semicolon]),
                    });
                }
            };

            return Ok(Statement::EnumDefinition { name, variants });
        }
        (t, _) => t,
    };

    let name = Identifier::parse(tokens)?;

    let array_peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
        ctx: EOFContext::Statement,
    })?;
    let f_type = match &array_peeked.data {
        TokenData::OpenBracket => {
            let _ = tokens.next();

            let tmp_peeked = tokens.peek();
            match tmp_peeked {
                Some(tok) if tok.data == TokenData::CloseBracket => {
                    let _ = tokens.next();

                    TypeToken::ArrayType {
                        base: Box::new(ty_tokens),
                        size: None,
                    }
                }
                _ => {
                    let size_exp = Expression::parse(tokens)?;

                    let close_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                        ctx: EOFContext::Statement,
                    })?;
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

            let mut var_args = false;
            let mut arguments: Vec<SpanData<FunctionArgument>> = Vec::new();
            while let Some(tmp_tok) = tokens.peek() {
                match &tmp_tok.data {
                    TokenData::CloseParen => {
                        let _ = tokens.next();
                        break;
                    }
                    TokenData::VarArgs => {
                        let _ = tokens.next();

                        var_args = true;

                        let close_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                            ctx: EOFContext::Statement,
                        })?;
                        match close_token.data {
                            TokenData::CloseParen => {
                                break;
                            }
                            _ => {
                                return Err(SyntaxError::UnexpectedToken {
                                    got: close_token.span,
                                    expected: Some(vec![ExpectedToken::CloseParen]),
                                });
                            }
                        };
                    }
                    _ => {}
                };

                let source = tmp_tok.span.source().clone();
                let start = tmp_tok.span.source_area().start;

                let ty = TypeToken::parse(tokens)?;

                let name_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
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

                let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
                if peeked.data == TokenData::Comma {
                    let _ = tokens.next();
                }
            }

            let f_head = FunctionHead {
                name,
                r_type: f_type,
                arguments,
                var_args,
            };

            let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                ctx: EOFContext::Statement,
            })?;
            match &next_tok.data {
                TokenData::OpenBrace => {
                    let inner_scope = Scope::parse(tokens)?;

                    Ok(Statement::FunctionDefinition {
                        head: f_head,
                        body: inner_scope,
                    })
                }
                TokenData::Semicolon => Ok(Statement::FunctionDeclaration(f_head)),
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

            let tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                ctx: EOFContext::Statement,
            })?;

            let exp = Expression::parse(tokens).map_err(|e| match e {
                SyntaxError::UnexpectedEOF { .. } => SyntaxError::ExpectedExpression {
                    span: tok.span,
                    reason: ExpressionReason::Assignment,
                },
                other => other,
            })?;

            let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                ctx: EOFContext::Statement,
            })?;
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
