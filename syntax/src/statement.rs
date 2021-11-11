use std::iter::Peekable;

use tokenizer::{ControlFlow, Keyword, Token, TokenData};

use crate::{
    expression::Expression, ExpressionOperator, FunctionArgument, Identifier, Scope,
    SingleOperation, SyntaxError, TypeToken,
};

mod starting_type;

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
    VariableDeclaration {
        ty: TypeToken,
        name: Identifier,
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
            TokenData::Keyword(Keyword::DataType(_)) => starting_type::parse(tokens),
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
                match next_token.data {
                    TokenData::Assign(assign_type) => {
                        let base_exp = Expression::parse(tokens)?;

                        let next_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                        match next_token.data {
                            TokenData::Semicolon => {}
                            other => panic!("Expected ';' but got '{:?}'", other),
                        };

                        let combine_op = ExpressionOperator::try_from(assign_type);
                        let exp = match combine_op {
                            Ok(op) => Expression::Operation {
                                left: Box::new(Expression::Identifier {
                                    ident: name.clone(),
                                }),
                                operation: op,
                                right: Box::new(base_exp),
                            },
                            Err(_) => base_exp,
                        };

                        Ok(Self::VariableAssignment { name, value: exp })
                    }
                    TokenData::OpenBracket => {
                        let index_exp = Expression::parse(tokens)?;

                        let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                        match next_tok.data {
                            TokenData::CloseBracket => {}
                            other => panic!("Expected ']' but got '{:?}'", other),
                        };

                        let assign_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                        let assign_type = match assign_token.data {
                            TokenData::Assign(a) => a,
                            other => panic!("Expected '=' or similiar but got '{:?}'", other),
                        };

                        let base_exp = Expression::parse(tokens)?;
                        let semi_colon_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                        match semi_colon_token.data {
                            TokenData::Semicolon => {}
                            other => panic!("Expected ';' but got '{:?}'", other),
                        };

                        let combine_op = ExpressionOperator::try_from(assign_type);

                        let value_exp = match combine_op {
                            Ok(op) => Expression::Operation {
                                left: Box::new(Expression::SingleOperation {
                                    base: Box::new(Expression::Identifier {
                                        ident: name.clone(),
                                    }),
                                    operation: SingleOperation::ArrayAccess(Box::new(
                                        index_exp.clone(),
                                    )),
                                }),
                                operation: op,
                                right: Box::new(base_exp),
                            },
                            Err(_) => base_exp,
                        };

                        Ok(Self::ArrayVariableAssignment {
                            name,
                            index: index_exp,
                            value: value_exp,
                        })
                    }
                    other => panic!("Expected '=' but got '{:?}'", other),
                }
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
    use general::{Span, SpanData};
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

        let result = Statement::parse(&mut input_tokens.peekable());

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

        let result = Statement::parse(&mut input_tokens.peekable());

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

        let result = Statement::parse(&mut input_tokens.peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn declare_array_known_size() {
        let input_content = "int test[3];";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Statement::VariableDeclaration {
            ty: TypeToken::ArrayType {
                base: Box::new(TypeToken::Primitive(SpanData {
                    span: Span::from_parts("test", "int", 0..3),
                    data: DataType::Int,
                })),
                size: Some(Box::new(Expression::Literal {
                    content: SpanData {
                        span: Span::from_parts("test", "3", 9..10),
                        data: "3".to_string(),
                    },
                })),
            },
            name: Identifier(SpanData {
                span: Span::from_parts("test", "test", 4..8),
                data: "test".to_string(),
            }),
        });

        let mut iter = input_tokens.peekable();
        let result = Statement::parse(&mut iter);

        assert_eq!(expected, result);
    }
    #[test]
    fn declare_array_unknown_size() {
        let input_content = "int test[];";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Statement::VariableDeclaration {
            ty: TypeToken::ArrayType {
                base: Box::new(TypeToken::Primitive(SpanData {
                    span: Span::from_parts("test", "int", 0..3),
                    data: DataType::Int,
                })),
                size: None,
            },
            name: Identifier(SpanData {
                span: Span::from_parts("test", "test", 4..8),
                data: "test".to_string(),
            }),
        });

        let mut iter = input_tokens.peekable();
        let result = Statement::parse(&mut iter);

        assert_eq!(expected, result);
    }

    #[test]
    fn declare_array_with_one_value() {
        let input_content = "int test[] = {1};";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Statement::VariableDeclarationAssignment {
            ty: TypeToken::ArrayType {
                base: Box::new(TypeToken::Primitive(SpanData {
                    span: Span::from_parts("test", "int", 0..3),
                    data: DataType::Int,
                })),
                size: None,
            },
            name: Identifier(SpanData {
                span: Span::from_parts("test", "test", 4..8),
                data: "test".to_string(),
            }),
            value: Expression::ArrayLiteral {
                parts: vec![Expression::Literal {
                    content: SpanData {
                        span: Span::from_parts("test", "1", 14..15),
                        data: "1".to_string(),
                    },
                }],
            },
        });

        let mut iter = input_tokens.peekable();
        let result = Statement::parse(&mut iter);

        assert_eq!(expected, result);
    }
    #[test]
    fn declare_array_with_two_values() {
        let input_content = "int test[] = {1, 2};";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Statement::VariableDeclarationAssignment {
            ty: TypeToken::ArrayType {
                base: Box::new(TypeToken::Primitive(SpanData {
                    span: Span::from_parts("test", "int", 0..3),
                    data: DataType::Int,
                })),
                size: None,
            },
            name: Identifier(SpanData {
                span: Span::from_parts("test", "test", 4..8),
                data: "test".to_string(),
            }),
            value: Expression::ArrayLiteral {
                parts: vec![
                    Expression::Literal {
                        content: SpanData {
                            span: Span::from_parts("test", "1", 14..15),
                            data: "1".to_string(),
                        },
                    },
                    Expression::Literal {
                        content: SpanData {
                            span: Span::from_parts("test", "2", 17..18),
                            data: "2".to_string(),
                        },
                    },
                ],
            },
        });

        let mut iter = input_tokens.peekable();
        let result = Statement::parse(&mut iter);

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

        let result = Statement::parse(&mut input_tokens.peekable());

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

        let result = Statement::parse(&mut input_tokens.peekable());

        assert_eq!(expected, result);
    }
}
