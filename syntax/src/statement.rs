use general::{Span, SpanData};
use itertools::PeekNth;
use tokenizer::{ControlFlow, DataType, Keyword, Operator, Token, TokenData};

use crate::{
    expression::Expression, EOFContext, ExpectedToken, FunctionArgument, Identifier, Scope,
    SyntaxError, TypeToken,
};

mod assign_target;
pub use assign_target::AssignTarget;

mod else_block;
mod starting_literal;
mod starting_type;

mod structs;
pub use structs::StructMembers;
mod enums;
pub use enums::*;

mod assign_type;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum TypeDefType {
    Type(TypeToken),
    NamedStruct {
        name: String,
    },
    StructdDef {
        name: Option<String>,
        members: structs::StructMembers,
        entire_span: Span,
    },
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct FunctionHead {
    pub r_type: TypeToken,
    pub name: Identifier,
    pub arguments: Vec<SpanData<FunctionArgument>>,
    pub var_args: bool,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
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
        /// The entire Span of the Struct Definition
        definition: Span,
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
        elses: Option<Scope>,
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
        let peeked = tokens.peek().ok_or_else(|| {
            println!("Peeked Parse Statement");

            SyntaxError::UnexpectedEOF {
                ctx: EOFContext::Statement,
            }
        })?;

        match &peeked.data {
            TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)) => {
                let _ = tokens.next();

                let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
                let exp = if is_termination(peeked.clone()).is_ok() {
                    let _ = tokens.next();
                    None
                } else {
                    let exp = Expression::parse(tokens)?;

                    let next_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                        ctx: EOFContext::Statement,
                    })?;
                    is_termination(next_token)?;

                    Some(exp)
                };

                Ok(Self::Return(exp))
            }
            TokenData::Literal { .. } => starting_literal::parse(tokens, is_termination),
            TokenData::Keyword(Keyword::DataType(_)) => {
                starting_type::parse(tokens, is_termination)
            }
            TokenData::Operator(tokenizer::Operator::Multiply) => {
                let _ = tokens.next();

                let target = Expression::parse(tokens)?;

                let assign_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
                let assign_type = match assign_token.data {
                    TokenData::Assign(as_ty) => as_ty,
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::Assignment]),
                            got: assign_token.span,
                        });
                    }
                };

                let raw_value = Expression::parse(tokens)?;

                let end_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
                is_termination(end_tok)?;

                let value = assign_type::convert_assign(raw_value, assign_type, || {
                    Expression::SingleOperation {
                        base: Box::new(target.clone()),
                        operation: crate::SingleOperation::Dereference,
                    }
                });

                Ok(Self::VariableDerefAssignment { target, value })
            }
            TokenData::Operator(Operator::Increment) | TokenData::Operator(Operator::Decrement) => {
                let exp = Expression::parse(tokens)?;

                let ending_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
                is_termination(ending_tok)?;

                Ok(Self::SingleExpression(exp))
            }
            TokenData::Comment { .. } => {
                todo!("Comments are not expected to be parsed as a Statement")
            }
            TokenData::Keyword(Keyword::TypeDef) => {
                let _ = tokens.next();

                let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
                match &peeked.data {
                    TokenData::Keyword(Keyword::DataType(DataType::Struct)) => {
                        let _ = tokens.next();

                        let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
                            ctx: EOFContext::Statement,
                        })?;
                        let (struct_name, start_span) = match &peeked.data {
                            TokenData::Literal { .. } => {
                                let next = tokens.next().expect("We just peeked it");
                                match next.data {
                                    TokenData::Literal { content } => (Some(content), next.span),
                                    _ => unreachable!("We previously matched on the Peeked Data and got a literal"),
                                }
                            }
                            TokenData::OpenBrace => (None, peeked.span.clone()),
                            _ => {
                                let next_tok = tokens.next().expect("We just peeked it");
                                return Err(SyntaxError::UnexpectedToken {
                                    expected: None,
                                    got: next_tok.span,
                                });
                            }
                        };

                        let members = structs::StructMembers::parse(tokens)?;

                        let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
                            ctx: EOFContext::Statement,
                        })?;
                        let end_span = peeked.span.clone();

                        let n_type_name = Identifier::parse(tokens)?;

                        let term_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                            ctx: EOFContext::Statement,
                        })?;
                        is_termination(term_token)?;

                        let entire_span = Span::new_arc_source(
                            start_span.source().clone(),
                            start_span.source_area().start..end_span.source_area().end,
                        );

                        Ok(Self::TypeDef {
                            name: n_type_name,
                            base_type: TypeDefType::StructdDef {
                                name: struct_name,
                                members,
                                entire_span,
                            },
                        })
                    }
                    _ => {
                        let ty = TypeToken::parse(tokens)?;

                        let n_type_name = Identifier::parse(tokens)?;

                        Ok(Self::TypeDef {
                            name: n_type_name,
                            base_type: TypeDefType::Type(ty),
                        })
                    }
                }
            }
            TokenData::Keyword(Keyword::ControlFlow(ControlFlow::If)) => {
                let _ = tokens.next();

                let open_paren_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
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

                let close_paren_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
                match close_paren_token.data {
                    TokenData::CloseParen => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::CloseParen]),
                            got: close_paren_token.span,
                        })
                    }
                };

                let open_brace_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
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

                let else_block = if let Some(peeked) = tokens.peek() {
                    match &peeked.data {
                        TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Else)) => {
                            tokens.next();

                            let else_scope = else_block::parse(tokens)?;
                            Some(else_scope)
                        }
                        _ => None,
                    }
                } else {
                    None
                };

                Ok(Statement::If {
                    condition: condition_exp,
                    scope: inner_scope,
                    elses: else_block,
                })
            }
            TokenData::Keyword(Keyword::ControlFlow(ControlFlow::While)) => {
                let _ = tokens.next();

                let open_paren_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
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

                let close_paren_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
                match close_paren_token.data {
                    TokenData::CloseParen => {}
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            expected: Some(vec![ExpectedToken::CloseParen]),
                            got: close_paren_token.span,
                        })
                    }
                };

                let open_brace_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
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

                let semi_colon_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
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

                let semi_colon_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
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

                let open_paren_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
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

                let semi_colon_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
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

                let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;

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
            _ => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: None,
                    got: tokens.next().unwrap().span,
                });
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
            definition: Span::new_source(source.clone(), 12..71),
        });

        let result = Statement::parse(&mut input_tokens, &Statement::default_terminaton());
        dbg!(&result);

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
