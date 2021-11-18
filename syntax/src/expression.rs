// Operators in C:
// * https://www.tutorialspoint.com/cprogramming/c_operators_precedence.htm
// * https://en.cppreference.com/w/c/language/operator_precedence

use general::SpanData;
use itertools::PeekNth;
use tokenizer::{Assignment, Operator, Token, TokenData};

use crate::{Identifier, SyntaxError, TypeToken};

mod parse_state;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier {
        ident: Identifier,
    },
    /// This could both represent a value like "0" as well as a Variable name
    Literal {
        content: SpanData<String>,
    },
    /// This represents a string Literal
    StringLiteral {
        content: SpanData<String>,
    },
    ArrayLiteral {
        parts: Vec<Expression>,
    },
    /// A Type-Cast
    Cast {
        /// The Target Type to Cast to
        target_ty: TypeToken,
        /// The Expression that should be casted
        exp: Box<Self>,
    },
    /// Some kind of Operation with only one operand
    SingleOperation {
        /// The Expression that this should be applied on
        base: Box<Self>,
        operation: SingleOperation,
    },
    /// Some kind of Operation like "+", "-", "<", "<<"
    Operation {
        /// The Left side of the Operation
        left: Box<Self>,
        /// The Operation that should be performed
        operation: ExpressionOperator,
        /// The Right side of the Operation
        right: Box<Self>,
    },
    Conditional {
        condition: Box<Self>,
        first: Box<Self>,
        second: Box<Self>,
    },
    StructAccess {
        base: Box<Self>,
        field: Identifier,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum SingleOperation {
    Positive,
    Negative,
    LogicalNot,
    BitwiseNot,
    PrefixIncrement,
    PrefixDecrement,
    Cast(TypeToken),
    Dereference,
    AddressOf,
    Sizeof,
    ArrayAccess(Box<Expression>),
    Arrow,
    Dot,
    SuffixIncrement,
    SuffixDecrement,
    FuntionCall(Vec<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionOperator {
    Add,
    Sub,
    Multiply,
    Divide,
    Modulo,
    LogicalAnd,
    LogicalOr,
    BitwiseXor,
    BitwiseAnd,
    BitwiseOr,
    ShiftLeft,
    ShiftRight,
    Equal,
    NotEqual,
    Less,
    Greater,
    GreaterEqual,
    LessEqual,
}

impl TryFrom<Assignment> for ExpressionOperator {
    type Error = ();

    fn try_from(value: Assignment) -> Result<Self, Self::Error> {
        match value {
            Assignment::Assign => Err(()),
            Assignment::Add => Ok(ExpressionOperator::Add),
            Assignment::Sub => Ok(ExpressionOperator::Sub),
            Assignment::Multiply => Ok(ExpressionOperator::Multiply),
            Assignment::Divide => Ok(ExpressionOperator::Divide),
            Assignment::Modulo => Ok(ExpressionOperator::Modulo),
            Assignment::ShiftLeft => Ok(ExpressionOperator::ShiftLeft),
            Assignment::ShiftRight => Ok(ExpressionOperator::ShiftRight),
            Assignment::BitwiseOr => Ok(ExpressionOperator::BitwiseOr),
            Assignment::BitwiseAnd => Ok(ExpressionOperator::BitwiseAnd),
            Assignment::BitwiseXor => Ok(ExpressionOperator::BitwiseXor),
        }
    }
}

impl Expression {
    fn parse_single_token(current: Token) -> Result<Self, SyntaxError> {
        match current.data {
            TokenData::Literal { content } => {
                match Identifier::from_literal(current.span.clone(), content.clone()) {
                    Ok(ident) => Ok(Self::Identifier { ident }),
                    Err(_) => Ok(Self::Literal {
                        content: SpanData {
                            span: current.span,
                            data: content,
                        },
                    }),
                }
            }
            TokenData::StringLiteral { content } => Ok(Self::StringLiteral {
                content: SpanData {
                    span: current.span,
                    data: content,
                },
            }),
            _ => Err(SyntaxError::UnexpectedToken {
                expected: None,
                got: current.span,
            }),
        }
    }

    fn parse_exp_list<I>(
        tokens: &mut PeekNth<I>,
        end_tok: TokenData,
    ) -> Result<Vec<Self>, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let mut result = Vec::new();

        while let Some(peeked) = tokens.peek() {
            if &peeked.data == &end_tok {
                break;
            }

            let tmp_exp = Self::parse(tokens)?;
            result.push(tmp_exp);

            let comma_ending_token = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;
            match &comma_ending_token.data {
                TokenData::Comma => {
                    let _ = tokens.next();
                }
                other if other == &end_tok => {}
                _ => {
                    let tok = tokens.next().expect("We already peeked it");
                    return Err(SyntaxError::UnexpectedToken {
                        expected: None,
                        got: tok.span,
                    });
                }
            };
        }

        Ok(result)
    }

    fn parse_expressions<I, F>(
        tokens: &mut PeekNth<I>,
        is_terminator: F,
    ) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
        F: Fn(&TokenData) -> bool,
    {
        // This basically builds up the Expression using Reverse Polish Notation with the
        // shunting-yard-algorithm
        let mut state = parse_state::ParseState::new();

        while let Some(peeked) = tokens.peek() {
            if is_terminator(&peeked.data) {
                break;
            }

            let current = tokens.next().unwrap();
            let new_last_data = current.data.clone();

            match &current.data {
                TokenData::Literal { .. } => {
                    let entry = Self::parse_single_token(current)?;

                    state.add_expression(entry);
                }
                TokenData::StringLiteral { .. } => {
                    let entry = Self::parse_single_token(current)?;

                    state.add_expression(entry);
                }
                TokenData::Operator(op) => {
                    match op {
                        Operator::BitwiseAnd => {
                            match state.get_cloned_last_token_data() {
                                Some(TokenData::Operator(_)) | None => {
                                    state.add_single_op(SingleOperation::AddressOf);
                                }
                                _ => {
                                    state.add_operator(&Operator::BitwiseAnd);
                                }
                            };
                        }
                        other => {
                            state.add_operator(other);
                        }
                    };
                }
                TokenData::OpenParen => {
                    match state.get_cloned_last_token_data() {
                        Some(TokenData::Literal { .. }) => {
                            dbg!("Got Function Call");

                            let params = Self::parse_exp_list(tokens, TokenData::CloseParen)?;

                            let closing_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                            match closing_token.data {
                                TokenData::CloseParen => {}
                                other => panic!("Expected ')' but got '{:?}'", other),
                            };

                            state.add_single_op(SingleOperation::FuntionCall(params));
                        }
                        _ => {
                            let exp = Self::parse(tokens)?;

                            let closing_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                            match closing_token.data {
                                TokenData::CloseParen => {}
                                _ => {
                                    return Err(SyntaxError::UnexpectedToken {
                                        expected: Some(vec![")".to_string()]),
                                        got: closing_token.span,
                                    })
                                }
                            };

                            state.add_expression(exp);
                        }
                    };
                }
                TokenData::OpenBracket => {
                    let exp = Self::parse(tokens)?;

                    let ending_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                    match ending_token.data {
                        TokenData::CloseBracket => {}
                        other => panic!("Expected ']' but got '{:?}'", other),
                    };

                    state.add_single_op(SingleOperation::ArrayAccess(Box::new(exp)));
                }
                TokenData::OpenBrace => {
                    let mut items = Vec::new();
                    while let Ok(exp) = Expression::parse(tokens) {
                        items.push(exp);

                        let peeked_seperator_token =
                            tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;
                        match &peeked_seperator_token.data {
                            TokenData::Comma => {
                                let _ = tokens.next();
                            }
                            TokenData::CloseBrace => {}
                            _ => {
                                let tmp = tokens.next().unwrap();
                                return Err(SyntaxError::UnexpectedToken {
                                    expected: Some(vec![",".to_string(), "}".to_string()]),
                                    got: tmp.span,
                                });
                            }
                        };
                    }

                    let closing_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                    match closing_token.data {
                        TokenData::CloseBrace => {}
                        _ => {
                            return Err(SyntaxError::UnexpectedToken {
                                expected: Some(vec!["}".to_string()]),
                                got: closing_token.span,
                            })
                        }
                    };

                    state.add_expression(Expression::ArrayLiteral { parts: items });
                }
                TokenData::QuestionMark => {
                    let first = Self::parse(tokens)?;
                    dbg!(&first);

                    let seperator_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                    match seperator_token.data {
                        TokenData::Colon => {}
                        _ => {
                            return Err(SyntaxError::UnexpectedToken {
                                expected: Some(vec![":".to_string()]),
                                got: seperator_token.span,
                            })
                        }
                    };

                    let second = Self::parse(tokens)?;
                    dbg!(&second);

                    state.add_conditional();
                    state.add_expression(first);
                    state.add_expression(second);
                }
                _ => {
                    return Err(SyntaxError::UnexpectedToken {
                        expected: None,
                        got: current.span,
                    })
                }
            };

            state.set_last_token_data(new_last_data);
        }

        let result = state.finalize().ok_or(SyntaxError::UnexpectedEOF)?;
        Ok(result)
    }

    pub fn parse<I>(tokens: &mut PeekNth<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        Self::parse_expressions(tokens, |data| match data {
            TokenData::Comma
            | TokenData::Semicolon
            | TokenData::CloseParen
            | TokenData::CloseBrace
            | TokenData::CloseBracket
            | TokenData::Colon => true,
            _ => false,
        })
    }
}

#[cfg(test)]
mod tests {
    use general::{Source, Span};
    use itertools::peek_nth;

    use super::*;

    #[test]
    fn empty() {
        let input_content = "";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Err(SyntaxError::UnexpectedEOF);

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(expected, result);
    }

    #[test]
    fn one_literal() {
        let input_content = "123";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::Literal {
            content: SpanData {
                span: Span::new_source(source.clone(), 0..3),
                data: "123".to_owned(),
            },
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(expected, result);
    }

    #[test]
    fn string_literal() {
        let input_content = "\"123\"";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::StringLiteral {
            content: SpanData {
                span: Span::new_source(source.clone(), 1..4),
                data: "123".to_owned(),
            },
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(expected, result);
    }

    #[test]
    fn add_two_literals() {
        let input_content = "123 + 234";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::Operation {
            left: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 0..3),
                    data: "123".to_owned(),
                },
            }),
            operation: ExpressionOperator::Add,
            right: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 6..9),
                    data: "234".to_owned(),
                },
            }),
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(expected, result);
    }
    #[test]
    fn add_three_literals() {
        let input_content = "123 + 234 + 345";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::Operation {
            left: Box::new(Expression::Operation {
                left: Box::new(Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 0..3),
                        data: "123".to_owned(),
                    },
                }),
                operation: ExpressionOperator::Add,
                right: Box::new(Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 6..9),
                        data: "234".to_owned(),
                    },
                }),
            }),
            operation: ExpressionOperator::Add,
            right: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 12..15),
                    data: "345".to_owned(),
                },
            }),
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(expected, result);
    }

    #[test]
    fn logical_not() {
        let input_content = "!123";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::SingleOperation {
            operation: SingleOperation::LogicalNot,
            base: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 1..4),
                    data: "123".to_owned(),
                },
            }),
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(expected, result);
    }
    #[test]
    fn logical_not_and_add() {
        let input_content = "!123 + 234";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::Operation {
            left: Box::new(Expression::SingleOperation {
                operation: SingleOperation::LogicalNot,
                base: Box::new(Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 1..4),
                        data: "123".to_owned(),
                    },
                }),
            }),
            operation: ExpressionOperator::Add,
            right: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 7..10),
                    data: "234".to_string(),
                },
            }),
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(expected, result);
    }

    #[test]
    fn array_access() {
        let input_content = "test[0]";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::SingleOperation {
            base: Box::new(Expression::Identifier {
                ident: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 0..4),
                    data: "test".to_string(),
                }),
            }),
            operation: SingleOperation::ArrayAccess(Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 5..6),
                    data: "0".to_string(),
                },
            })),
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn empty_function_call() {
        let input_content = "test()";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::SingleOperation {
            base: Box::new(Expression::Identifier {
                ident: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 0..4),
                    data: "test".to_string(),
                }),
            }),
            operation: SingleOperation::FuntionCall(vec![]),
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }
    #[test]
    fn one_arg_function_call() {
        let input_content = "test(0)";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::SingleOperation {
            base: Box::new(Expression::Identifier {
                ident: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 0..4),
                    data: "test".to_string(),
                }),
            }),
            operation: SingleOperation::FuntionCall(vec![Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 5..6),
                    data: "0".to_string(),
                },
            }]),
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }
    #[test]
    fn two_args_function_call() {
        let input_content = "test(0,1)";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::SingleOperation {
            base: Box::new(Expression::Identifier {
                ident: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 0..4),
                    data: "test".to_string(),
                }),
            }),
            operation: SingleOperation::FuntionCall(vec![
                Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 5..6),
                        data: "0".to_string(),
                    },
                },
                Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 7..8),
                        data: "1".to_string(),
                    },
                },
            ]),
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn array_literal_empty() {
        let input_content = "{}";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::ArrayLiteral { parts: Vec::new() });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn function_call_2_args() {
        let input_content = "test(2, 3)";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::SingleOperation {
            base: Box::new(Expression::Identifier {
                ident: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 0..4),
                    data: "test".to_string(),
                }),
            }),
            operation: SingleOperation::FuntionCall(vec![
                Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 5..6),
                        data: "2".to_string(),
                    },
                },
                Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 8..9),
                        data: "3".to_string(),
                    },
                },
            ]),
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn conditional_simple() {
        let input_content = "1 ? 2 : 3";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::Conditional {
            condition: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 0..1),
                    data: "1".to_string(),
                },
            }),
            first: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 4..5),
                    data: "2".to_string(),
                },
            }),
            second: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 8..9),
                    data: "3".to_string(),
                },
            }),
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn parens() {
        let input_content = "(1 + 2)";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(Expression::Operation {
            operation: ExpressionOperator::Add,
            left: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 1..2),
                    data: "1".to_string(),
                },
            }),
            right: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 5..6),
                    data: "2".to_string(),
                },
            }),
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }
}
