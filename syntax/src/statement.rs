use general::SpanData;
use itertools::PeekNth;
use tokenizer::{ControlFlow, DataType, Keyword, Token, TokenData};

use crate::{
    expression::Expression, ExpectedToken, FunctionArgument, Identifier, Scope, SyntaxError,
    TypeToken,
};

mod assign_target;
pub use assign_target::AssignTarget;

mod starting_literal;
mod starting_type;

mod structs;
pub use structs::StructMembers;
mod enums;
pub use enums::*;

#[derive(Debug, PartialEq)]
pub enum TypeDefType {
    Type(TypeToken),
    NamedStruct {
        name: String,
    },
    StructdDef {
        name: Option<String>,
        members: structs::StructMembers,
    },
}

#[derive(Debug, PartialEq)]
pub struct FunctionHead {
    pub r_type: TypeToken,
    pub name: Identifier,
    pub arguments: Vec<SpanData<FunctionArgument>>,
    pub var_args: bool,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    SubScope(Scope),
    FunctionDeclaration(FunctionHead),
    FunctionDefinition {
        head: FunctionHead,
        body: Scope,
    },
    StructDefinition {
        /// The Name of the Struct
        name: Identifier,
        /// The members of the Struct
        members: StructMembers,
    },
    EnumDefinition {
        name: Identifier,
        variants: EnumVariants,
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
        target: AssignTarget,
        value: Expression,
    },
    VariableDerefAssignment {
        target: Expression,
        value: Expression,
    },
    TypeDef {
        name: Identifier,
        base_type: TypeDefType,
    },
    SingleExpression(Expression),
    WhileLoop {
        condition: Expression,
        scope: Scope,
    },
    ForLoop {
        setup: Vec<Self>,
        condition: Expression,
        update: Vec<Self>,
        scope: Scope,
    },
    If {
        condition: Expression,
        scope: Scope,
        elses: Vec<(Option<Expression>, Scope)>,
    },
    Continue,
    Break,
    Return(Option<Expression>),
}

impl Statement {
    pub fn default_terminaton() -> impl Fn(Token) -> Result<(), SyntaxError> {
        |token: Token| match token.data {
            TokenData::Semicolon => Ok(()),
            _ => Err(SyntaxError::UnexpectedToken {
                expected: Some(vec![ExpectedToken::Semicolon]),
                got: token.span,
            }),
        }
    }

    pub fn parse<I>(
        tokens: &mut PeekNth<I>,
        is_termination: &dyn Fn(Token) -> Result<(), SyntaxError>,
    ) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;

        match &peeked.data {
            TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)) => {
                let _ = tokens.next();

                let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;
                let exp = if is_termination(peeked.clone()).is_ok() {
                    let _ = tokens.next();
                    None
                } else {
                    let exp = Expression::parse(tokens)?;

                    let next_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                    is_termination(next_token)?;

                    Some(exp)
                };

                Ok(Self::Return(exp))
            }
            TokenData::Literal { .. } => starting_literal::parse(tokens, is_termination),
            TokenData::Keyword(Keyword::DataType(_)) => {
                starting_type::parse(tokens, is_termination)
            }
            TokenData::Comment { .. } => {
                todo!("Comments are not expected to be parsed as a Statement")
            }
            TokenData::Keyword(Keyword::TypeDef) => {
                let _ = tokens.next();

                let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;
                match &peeked.data {
                    TokenData::Keyword(Keyword::DataType(DataType::Struct)) => {
                        let _ = tokens.next();

                        let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;
                        let struct_name = match &peeked.data {
                            TokenData::Literal { .. } => {
                                let next = tokens.next().expect("We just peeked it");
                                match next.data {
                                    TokenData::Literal { content } => Some(content),
                                    _ => unreachable!("We previously matched on the Peeked Data and got a literal"),
                                }
                            }
                            TokenData::OpenBrace => None,
                            _ => {
                                let next_tok = tokens.next().expect("We just peeked it");
                                return Err(SyntaxError::UnexpectedToken {
                                    expected: None,
                                    got: next_tok.span,
                                });
                            }
                        };

                        let members = structs::StructMembers::parse(tokens)?;

                        let n_type_name = Identifier::parse(tokens)?;

                        let term_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                        is_termination(term_token)?;

                        Ok(Self::TypeDef {
                            name: n_type_name,
                            base_type: TypeDefType::StructdDef {
                                name: struct_name,
                                members,
                            },
                        })
                    }
                    _ => {
                        let ty = TypeToken::parse(tokens)?;

                        dbg!(&ty);

                        todo!("Parsing TypeDef");
                    }
                }
            }
            TokenData::Keyword(Keyword::ControlFlow(ControlFlow::If)) => {
                let _ = tokens.next();

                let open_paren_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match open_paren_token.data {
                    TokenData::OpenParen => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::OpenParen]),
                            got: open_paren_token.span,
                        })
                    }
                };

                let condition_exp = Expression::parse(tokens)?;

                let close_paren_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match close_paren_token.data {
                    TokenData::CloseParen => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::CloseParen]),
                            got: close_paren_token.span,
                        })
                    }
                };

                let open_brace_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match open_brace_token.data {
                    TokenData::OpenBrace => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::OpenBrace]),
                            got: open_brace_token.span,
                        })
                    }
                };

                let inner_scope = Scope::parse(tokens)?;

                dbg!(&condition_exp, &inner_scope);

                let mut elses = Vec::new();
                while let Some(peeked) = tokens.peek() {
                    match &peeked.data {
                        TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Else)) => {}
                        _ => break,
                    };

                    // Consume the Else Token
                    let _ = tokens.next();

                    let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                    match next_tok.data {
                        TokenData::OpenBrace => {
                            let scope = Scope::parse(tokens)?;

                            elses.push((None, scope));
                        }
                        TokenData::Keyword(Keyword::ControlFlow(ControlFlow::If)) => {
                            todo!("Conditional Else");
                        }
                        _ => {
                            return Err(SyntaxError::UnexpectedToken {
                                expected: Some(vec![ExpectedToken::OpenBrace, ExpectedToken::If]),
                                got: next_tok.span,
                            })
                        }
                    }
                }

                Ok(Statement::If {
                    condition: condition_exp,
                    scope: inner_scope,
                    elses,
                })
            }
            TokenData::Keyword(Keyword::ControlFlow(ControlFlow::While)) => {
                let _ = tokens.next();

                let open_paren_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match open_paren_token.data {
                    TokenData::OpenParen => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::OpenParen]),
                            got: open_paren_token.span,
                        })
                    }
                };

                let condition_exp = Expression::parse(tokens)?;

                let close_paren_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match close_paren_token.data {
                    TokenData::CloseParen => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::CloseParen]),
                            got: close_paren_token.span,
                        })
                    }
                };

                let open_brace_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match open_brace_token.data {
                    TokenData::OpenBrace => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::OpenBrace]),
                            got: open_brace_token.span,
                        })
                    }
                };

                let inner_scope = Scope::parse(tokens)?;

                Ok(Self::WhileLoop {
                    condition: condition_exp,
                    scope: inner_scope,
                })
            }
            TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Break)) => {
                let _ = tokens.next();

                let semi_colon_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match semi_colon_token.data {
                    TokenData::Semicolon => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::Semicolon]),
                            got: semi_colon_token.span,
                        })
                    }
                };

                Ok(Statement::Break)
            }
            TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Continue)) => {
                let _ = tokens.next();

                let semi_colon_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match semi_colon_token.data {
                    TokenData::Semicolon => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::Semicolon]),
                            got: semi_colon_token.span,
                        })
                    }
                };

                Ok(Statement::Continue)
            }
            TokenData::Keyword(Keyword::ControlFlow(ControlFlow::For)) => {
                let _ = tokens.next();

                let open_paren_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match open_paren_token.data {
                    TokenData::OpenParen => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::OpenParen]),
                            got: open_paren_token.span,
                        })
                    }
                };

                let init_statement = Self::parse(tokens, &Self::default_terminaton())?;

                let cond_exp = Expression::parse(tokens)?;

                let semi_colon_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                match semi_colon_token.data {
                    TokenData::Semicolon => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::Semicolon]),
                            got: semi_colon_token.span,
                        })
                    }
                };

                let post_statement_termination = |token: Token| match token.data {
                    TokenData::CloseParen => Ok(()),
                    _ => Err(SyntaxError::UnexpectedToken {
                        expected: Some(vec![ExpectedToken::OpenParen]),
                        got: token.span,
                    }),
                };
                let post_statement = Self::parse(tokens, &post_statement_termination)?;

                let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;

                match &peeked.data {
                    TokenData::OpenBrace => {
                        let _ = tokens.next();
                    }
                    _ => {
                        let tmp = tokens.next().unwrap();
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::OpenBrace]),
                            got: tmp.span,
                        });
                    }
                };

                let inner_scope = Scope::parse(tokens)?;

                Ok(Self::ForLoop {
                    setup: vec![init_statement],
                    condition: cond_exp,
                    update: vec![post_statement],
                    scope: inner_scope,
                })
            }
            unknown => {
                todo!("Parse: {:?}", unknown);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use general::{Source, Span, SpanData};
    use itertools::peek_nth;
    use tokenizer::DataType;

    use crate::{ExpressionOperator, SingleOperation};

    use super::*;

    #[test]
    fn function_definition_no_args() {
        let input_content = "int test() {}";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::FunctionDefinition {
            head: FunctionHead {
                r_type: TypeToken::Primitive(SpanData {
                    span: Span::new_source(source.clone(), 0..3),
                    data: DataType::Int,
                }),
                name: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 4..8),
                    data: "test".to_string(),
                }),
                arguments: Vec::new(),
                var_args: false,
            },
            body: Scope {
                statements: Vec::new(),
            },
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }

    #[test]
    fn function_definition_one_arg() {
        let input_content = "int test(int x) {}";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::FunctionDefinition {
            head: FunctionHead {
                r_type: TypeToken::Primitive(SpanData {
                    span: Span::new_source(source.clone(), 0..3),
                    data: DataType::Int,
                }),
                name: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 4..8),
                    data: "test".to_string(),
                }),
                arguments: vec![SpanData {
                    span: Span::new_source(source.clone(), 9..14),
                    data: FunctionArgument {
                        name: Identifier(SpanData {
                            span: Span::new_source(source.clone(), 13..14),
                            data: "x".to_string(),
                        }),
                        ty: TypeToken::Primitive(SpanData {
                            span: Span::new_source(source.clone(), 9..12),
                            data: DataType::Int,
                        }),
                    },
                }],
                var_args: false,
            },
            body: Scope {
                statements: Vec::new(),
            },
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }

    #[test]
    fn define_struct() {
        let input_content = "struct test {
            int first;
            int second;
        };";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::StructDefinition {
            name: Identifier(SpanData {
                span: Span::new_source(source.clone(), 7..11),
                data: "test".to_string(),
            }),
            members: structs::StructMembers {
                members: vec![
                    (
                        TypeToken::Primitive(SpanData {
                            span: Span::new_source(source.clone(), 26..29),
                            data: DataType::Int,
                        }),
                        Identifier(SpanData {
                            span: Span::new_source(source.clone(), 30..35),
                            data: "first".to_string(),
                        }),
                    ),
                    (
                        TypeToken::Primitive(SpanData {
                            span: Span::new_source(source.clone(), 49..52),
                            data: DataType::Int,
                        }),
                        Identifier(SpanData {
                            span: Span::new_source(source.clone(), 53..59),
                            data: "second".to_string(),
                        }),
                    ),
                ],
            },
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }

    #[test]
    fn declare_var_primitive() {
        let input_content = "int test;";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::VariableDeclaration {
            ty: TypeToken::Primitive(SpanData {
                span: Span::new_source(source.clone(), 0..3),
                data: DataType::Int,
            }),
            name: Identifier(SpanData {
                span: Span::new_source(source.clone(), 4..8),
                data: "test".to_string(),
            }),
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn declare_var_custom_type() {
        let input_content = "Rect test;";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::VariableDeclaration {
            ty: TypeToken::TypeDefed {
                name: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 0..4),
                    data: "Rect".to_string(),
                }),
            },
            name: Identifier(SpanData {
                span: Span::new_source(source.clone(), 5..9),
                data: "test".to_string(),
            }),
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }
    #[test]
    fn declare_var_custom_type_ptr() {
        let input_content = "Rect* test;";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::VariableDeclaration {
            ty: TypeToken::Pointer(Box::new(TypeToken::TypeDefed {
                name: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 0..4),
                    data: "Rect".to_string(),
                }),
            })),
            name: Identifier(SpanData {
                span: Span::new_source(source.clone(), 6..10),
                data: "test".to_string(),
            }),
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn declare_array_known_size() {
        let input_content = "int test[3];";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::VariableDeclaration {
            ty: TypeToken::ArrayType {
                base: Box::new(TypeToken::Primitive(SpanData {
                    span: Span::new_source(source.clone(), 0..3),
                    data: DataType::Int,
                })),
                size: Some(Box::new(Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 9..10),
                        data: "3".to_string(),
                    },
                })),
            },
            name: Identifier(SpanData {
                span: Span::new_source(source.clone(), 4..8),
                data: "test".to_string(),
            }),
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }
    #[test]
    fn declare_array_unknown_size() {
        let input_content = "int test[];";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::VariableDeclaration {
            ty: TypeToken::ArrayType {
                base: Box::new(TypeToken::Primitive(SpanData {
                    span: Span::new_source(source.clone(), 0..3),
                    data: DataType::Int,
                })),
                size: None,
            },
            name: Identifier(SpanData {
                span: Span::new_source(source.clone(), 4..8),
                data: "test".to_string(),
            }),
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }

    #[test]
    fn declare_array_with_one_value() {
        let input_content = "int test[] = {1};";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::VariableDeclarationAssignment {
            ty: TypeToken::ArrayType {
                base: Box::new(TypeToken::Primitive(SpanData {
                    span: Span::new_source(source.clone(), 0..3),
                    data: DataType::Int,
                })),
                size: None,
            },
            name: Identifier(SpanData {
                span: Span::new_source(source.clone(), 4..8),
                data: "test".to_string(),
            }),
            value: Expression::ArrayLiteral {
                parts: SpanData {
                    span: Span::new_source(source.clone(), 13..16),
                    data: vec![Expression::Literal {
                        content: SpanData {
                            span: Span::new_source(source.clone(), 14..15),
                            data: "1".to_string(),
                        },
                    }],
                },
            },
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }
    #[test]
    fn declare_array_with_two_values() {
        let input_content = "int test[] = {1, 2};";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::VariableDeclarationAssignment {
            ty: TypeToken::ArrayType {
                base: Box::new(TypeToken::Primitive(SpanData {
                    span: Span::new_source(source.clone(), 0..3),
                    data: DataType::Int,
                })),
                size: None,
            },
            name: Identifier(SpanData {
                span: Span::new_source(source.clone(), 4..8),
                data: "test".to_string(),
            }),
            value: Expression::ArrayLiteral {
                parts: SpanData {
                    span: Span::new_source(source.clone(), 13..19),
                    data: vec![
                        Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(source.clone(), 14..15),
                                data: "1".to_string(),
                            },
                        },
                        Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(source.clone(), 17..18),
                                data: "2".to_string(),
                            },
                        },
                    ],
                },
            },
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }

    #[test]
    fn variable_assignment() {
        let input_content = "test = 13;";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::VariableAssignment {
            target: AssignTarget::Variable(Identifier(SpanData {
                span: Span::new_source(source.clone(), 0..4),
                data: "test".to_string(),
            })),
            value: Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 7..9),
                    data: "13".to_string(),
                },
            },
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }
    #[test]
    fn variable_array_assignment() {
        let input_content = "test[0] = 13;";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::VariableAssignment {
            target: AssignTarget::ArrayAccess {
                base: Box::new(AssignTarget::Variable(Identifier(SpanData {
                    span: Span::new_source(source.clone(), 0..4),
                    data: "test".to_string(),
                }))),
                index: Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 5..6),
                        data: "0".to_string(),
                    },
                },
            },
            value: Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 10..12),
                    data: "13".to_string(),
                },
            },
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }

    #[test]
    fn funcion_call_noargs() {
        let input_content = "test();";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::SingleExpression(Expression::SingleOperation {
            base: Box::new(Expression::Identifier {
                ident: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 0..4),
                    data: "test".to_string(),
                }),
            }),
            operation: SingleOperation::FuntionCall(vec![]),
        }));

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }
    #[test]
    fn funcion_call_1arg() {
        let input_content = "test(\"literal\");";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::SingleExpression(Expression::SingleOperation {
            base: Box::new(Expression::Identifier {
                ident: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 0..4),
                    data: "test".to_string(),
                }),
            }),
            operation: SingleOperation::FuntionCall(vec![Expression::StringLiteral {
                content: SpanData {
                    span: Span::new_source(source.clone(), 5..14),
                    data: "literal".to_string(),
                },
            }]),
        }));

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }

    #[test]
    fn return_with_paren() {
        let input_content = "return (1 + 2);";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::Return(Some(Expression::Operation {
            operation: ExpressionOperator::Add,
            left: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 8..9),
                    data: "1".to_string(),
                },
            }),
            right: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 12..13),
                    data: "2".to_string(),
                },
            }),
        })));

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }

    #[test]
    fn return_nothing() {
        let input_content = "return;";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Statement::Return(None));

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());

        assert_eq!(expected, result);
    }
}
