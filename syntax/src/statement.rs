use std::iter::Peekable;

use general::SpanData;
use tokenizer::{Assignment, ControlFlow, Keyword, Token, TokenData};

use crate::{
    expression::Expression, ExpressionOperator, FunctionArgument, Identifier, Scope, SyntaxError,
    TypeToken,
};

#[derive(Debug, PartialEq)]
pub enum Statement {
    SubScope(Scope),
    FunctionDefinition {
        /// The Return Type of the Function
        r_type: TypeToken,
        /// The Name of the Function
        name: Identifier,
        /// The Arguments to the Function
        arguments: Vec<FunctionArgument>,
        /// The Body of the Function
        body: Scope,
    },
    StructDefinition {
        /// The Name of the Struct
        name: Identifier,
        /// The members of the Struct
        members: Vec<(TypeToken, Identifier)>,
    },
    VariableDeclarationAssignment {
        ty: TypeToken,
        name: Identifier,
        value: Expression,
    },
    VariableAssignment {
        name: Identifier,
        value: Expression,
    },
    ArrayVariableAssignment {
        name: Identifier,
        index: Expression,
        value: Expression,
    },
    VariableDerefAssignment {
        target: Expression,
        value: Expression,
    },
    Return(Expression),
}

impl Statement {
    pub fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;

        match &peeked.data {
            TokenData::Keyword(Keyword::DataType(_)) => {
                let ty_tokens = TypeToken::parse(tokens)?;
                let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;
                let ty_tokens = match (ty_tokens, &peeked.data) {
                    (TypeToken::StructType { name }, TokenData::OpenBrace) => {
                        let _ = tokens.next();

                        let mut members = Vec::new();
                        while let Some(peeked) = tokens.peek() {
                            match &peeked.data {
                                TokenData::CloseBrace => break,
                                _ => {}
                            };

                            let field_ty = TypeToken::parse(tokens)?;
                            let field_ident = Identifier::parse(tokens)?;

                            let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                            match next_tok.data {
                                TokenData::Semicolon => {}
                                _ => {
                                    return Err(SyntaxError::UnexpectedToken {
                                        expected: Some(vec![";".to_string()]),
                                        got: next_tok.span,
                                    })
                                }
                            };

                            members.push((field_ty, field_ident));
                        }

                        return Ok(Self::StructDefinition { name, members });
                    }
                    (t, _) => t,
                };

                let name = Identifier::parse(tokens)?;

                let peeked = tokens.peek().unwrap();

                match &peeked.data {
                    TokenData::OpenParen => {
                        let _ = tokens.next();

                        let mut arguments: Vec<FunctionArgument> = Vec::new();
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

                            let ty = TypeToken::parse(tokens)?;

                            let name_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                            let name = match name_token.data {
                                TokenData::Literal { content } => Identifier(SpanData {
                                    span: name_token.span,
                                    data: content,
                                }),
                                _ => {
                                    return Err(SyntaxError::UnexpectedToken {
                                        expected: Some(vec!["Identifier".to_string()]),
                                        got: name_token.span,
                                    })
                                }
                            };

                            arguments.push(FunctionArgument { name, ty });
                        }

                        let next_tok = tokens.next().unwrap();
                        match &next_tok.data {
                            TokenData::OpenBrace => {
                                let inner_scope = Scope::parse(tokens);

                                Ok(Self::FunctionDefinition {
                                    name,
                                    r_type: ty_tokens,
                                    arguments,
                                    body: inner_scope,
                                })
                            }
                            TokenData::Semicolon => {
                                todo!("Parse Declaration");
                            }
                            other => panic!("Expected a {{ or ; but got: {:?}", other),
                        }
                    }
                    TokenData::Semicolon => {
                        todo!("Variable Declaration");
                    }
                    TokenData::Assign(assign_type) => {
                        match assign_type {
                            Assignment::Assign => {}
                            other => {
                                panic!("Expected a normal Assignment('=') but got '{}'", other)
                            }
                        };

                        let _ = tokens.next();

                        let exp = Expression::parse(tokens)?;

                        let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                        match next_tok.data {
                            TokenData::Semicolon => {}
                            _ => {
                                return Err(SyntaxError::UnexpectedToken {
                                    expected: Some(vec![";".to_owned()]),
                                    got: next_tok.span,
                                })
                            }
                        };

                        Ok(Self::VariableDeclarationAssignment {
                            ty: ty_tokens,
                            name,
                            value: exp,
                        })
                    }
                    tok_data => {
                        panic!("Unexpected Token: {:?}", tok_data);
                    }
                }
            }
            TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)) => {
                let _ = tokens.next();

                let exp = Expression::parse(tokens)?;

                let next_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match next_token.data {
                    TokenData::Semicolon => {}
                    other => panic!("Expected ';' but got '{:?}'", other),
                };

                Ok(Self::Return(exp))
            }
            TokenData::Literal { .. } => {
                let name = Identifier::parse(tokens)?;

                let next_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                let assign_type = match next_token.data {
                    TokenData::Assign(as_type) => as_type,
                    other => panic!("Expected '=' but got '{}'", other),
                };

                let base_exp = Expression::parse(tokens)?;

                let next_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match next_token.data {
                    TokenData::Semicolon => {}
                    other => panic!("Expected ';' but got '{:?}'", other),
                };

                let combine_op = match assign_type {
                    Assignment::Assign => None,
                    Assignment::Add => Some(ExpressionOperator::Add),
                    Assignment::Sub => Some(ExpressionOperator::Sub),
                    Assignment::Multiply => Some(ExpressionOperator::Multiply),
                    Assignment::Divide => Some(ExpressionOperator::Divide),
                    Assignment::Modulo => Some(ExpressionOperator::Modulo),
                    Assignment::ShiftLeft => Some(ExpressionOperator::ShiftLeft),
                    Assignment::ShiftRight => Some(ExpressionOperator::ShiftRight),
                    Assignment::BitwiseOr => Some(ExpressionOperator::BitwiseOr),
                    Assignment::BitwiseAnd => Some(ExpressionOperator::BitwiseAnd),
                    Assignment::BitwiseXor => Some(ExpressionOperator::BitwiseXor),
                };
                let exp = match combine_op {
                    Some(op) => Expression::Operation {
                        left: Box::new(Expression::Identifier {
                            ident: name.clone(),
                        }),
                        operation: op,
                        right: Box::new(base_exp),
                    },
                    None => base_exp,
                };

                Ok(Self::VariableAssignment { name, value: exp })
            }
            unknown => {
                dbg!(unknown);
                todo!("")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use general::Span;
    use tokenizer::DataType;

    use super::*;

    #[test]
    fn function_definition_no_args() {
        let input_content = "int test() {}";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Statement::FunctionDefinition {
            r_type: TypeToken::Primitive(SpanData {
                span: Span::from_parts("test", "int", 0..3),
                data: DataType::Int,
            }),
            name: Identifier(SpanData {
                span: Span::from_parts("test", "test", 4..8),
                data: "test".to_string(),
            }),
            arguments: Vec::new(),
            body: Scope {
                statements: Vec::new(),
            },
        });

        let result = Statement::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn function_definition_one_arg() {
        let input_content = "int test(int x) {}";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Statement::FunctionDefinition {
            r_type: TypeToken::Primitive(SpanData {
                span: Span::from_parts("test", "int", 0..3),
                data: DataType::Int,
            }),
            name: Identifier(SpanData {
                span: Span::from_parts("test", "test", 4..8),
                data: "test".to_string(),
            }),
            arguments: vec![FunctionArgument {
                name: Identifier(SpanData {
                    span: Span::from_parts("test", "x", 13..14),
                    data: "x".to_string(),
                }),
                ty: TypeToken::Primitive(SpanData {
                    span: Span::from_parts("test", "int", 9..12),
                    data: DataType::Int,
                }),
            }],
            body: Scope {
                statements: Vec::new(),
            },
        });

        let result = Statement::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn define_struct() {
        let input_content = "struct test {
            int first;
            int second;
        }";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Statement::StructDefinition {
            name: Identifier(SpanData {
                span: Span::from_parts("test", "test", 7..11),
                data: "test".to_string(),
            }),
            members: vec![
                (
                    TypeToken::Primitive(SpanData {
                        span: Span::from_parts("test", "int", 26..29),
                        data: DataType::Int,
                    }),
                    Identifier(SpanData {
                        span: Span::from_parts("test", "first", 30..35),
                        data: "first".to_string(),
                    }),
                ),
                (
                    TypeToken::Primitive(SpanData {
                        span: Span::from_parts("test", "int", 49..52),
                        data: DataType::Int,
                    }),
                    Identifier(SpanData {
                        span: Span::from_parts("test", "second", 53..59),
                        data: "second".to_string(),
                    }),
                ),
            ],
        });

        let result = Statement::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn variable_assignment() {
        let input_content = "test = 13;";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Statement::VariableAssignment {
            name: Identifier(SpanData {
                span: Span::from_parts("test", "test", 0..4),
                data: "test".to_string(),
            }),
            value: Expression::Literal {
                content: SpanData {
                    span: Span::from_parts("test", "13", 7..9),
                    data: "13".to_string(),
                },
            },
        });

        let result = Statement::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }
    #[test]
    fn variable_array_assignment() {
        let input_content = "test[0] = 13;";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Statement::ArrayVariableAssignment {
            name: Identifier(SpanData {
                span: Span::from_parts("test", "test", 0..4),
                data: "test".to_string(),
            }),
            index: Expression::Literal {
                content: SpanData {
                    span: Span::from_parts("test", "0", 5..6),
                    data: "0".to_string(),
                },
            },
            value: Expression::Literal {
                content: SpanData {
                    span: Span::from_parts("test", "13", 10..12),
                    data: "13".to_string(),
                },
            },
        });

        let result = Statement::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }
}
