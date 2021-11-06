// Operators in C:
// * https://www.tutorialspoint.com/cprogramming/c_operators_precedence.htm
// * https://en.cppreference.com/w/c/language/operator_precedence

use std::iter::Peekable;

use general::SpanData;
use tokenizer::{Assignment, Token, TokenData};

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
        tokens: &mut Peekable<I>,
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

    fn parse_expressions<I>(tokens: &mut Peekable<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        // This basically builds up the Expression using Reverse Polish Notation with the
        // shunting-yard-algorithm
        let mut state = parse_state::ParseState::new();

        while let Some(peeked) = tokens.peek() {
            match &peeked.data {
                TokenData::Comma => break,
                TokenData::Semicolon => break,
                TokenData::CloseParen => break,
                TokenData::CloseBrace => break,
                TokenData::CloseBracket => break,
                _ => {}
            };

            let current = tokens.next().unwrap();
            let new_last_data = current.data.clone();

            dbg!(&current.data);

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
                    state.add_operator(op);
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

                            let peeked = tokens.peek();
                            dbg!(peeked);

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

    pub fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        Self::parse_expressions(tokens)
    }
}

#[cfg(test)]
mod tests {
    use general::Span;

    use super::*;

    #[test]
    fn empty() {
        let input_content = "";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Err(SyntaxError::UnexpectedEOF);

        let result = Expression::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn one_literal() {
        let input_content = "123";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Expression::Literal {
            content: SpanData {
                span: Span::from_parts("test", "123", 0..3),
                data: "123".to_owned(),
            },
        });

        let result = Expression::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn string_literal() {
        let input_content = "\"123\"";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Expression::StringLiteral {
            content: SpanData {
                span: Span::from_parts("test", "123", 1..4),
                data: "123".to_owned(),
            },
        });

        let result = Expression::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn add_two_literals() {
        let input_content = "123 + 234";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Expression::Operation {
            left: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::from_parts("test", "123", 0..3),
                    data: "123".to_owned(),
                },
            }),
            operation: ExpressionOperator::Add,
            right: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::from_parts("test", "234", 6..9),
                    data: "234".to_owned(),
                },
            }),
        });

        let result = Expression::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }
    #[test]
    fn add_three_literals() {
        let input_content = "123 + 234 + 345";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Expression::Operation {
            left: Box::new(Expression::Operation {
                left: Box::new(Expression::Literal {
                    content: SpanData {
                        span: Span::from_parts("test", "123", 0..3),
                        data: "123".to_owned(),
                    },
                }),
                operation: ExpressionOperator::Add,
                right: Box::new(Expression::Literal {
                    content: SpanData {
                        span: Span::from_parts("test", "234", 6..9),
                        data: "234".to_owned(),
                    },
                }),
            }),
            operation: ExpressionOperator::Add,
            right: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::from_parts("test", "345", 12..15),
                    data: "345".to_owned(),
                },
            }),
        });

        let result = Expression::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn logical_not() {
        let input_content = "!123";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Expression::SingleOperation {
            operation: SingleOperation::LogicalNot,
            base: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::from_parts("test", "123", 1..4),
                    data: "123".to_owned(),
                },
            }),
        });

        let result = Expression::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }
    #[test]
    fn logical_not_and_add() {
        let input_content = "!123 + 234";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Expression::Operation {
            left: Box::new(Expression::SingleOperation {
                operation: SingleOperation::LogicalNot,
                base: Box::new(Expression::Literal {
                    content: SpanData {
                        span: Span::from_parts("test", "123", 1..4),
                        data: "123".to_owned(),
                    },
                }),
            }),
            operation: ExpressionOperator::Add,
            right: Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::from_parts("test", "234", 7..10),
                    data: "234".to_string(),
                },
            }),
        });

        let result = Expression::parse(&mut input_tokens.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn array_access() {
        let input_content = "test[0]";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Expression::SingleOperation {
            base: Box::new(Expression::Identifier {
                ident: Identifier(SpanData {
                    span: Span::from_parts("test", "test", 0..4),
                    data: "test".to_string(),
                }),
            }),
            operation: SingleOperation::ArrayAccess(Box::new(Expression::Literal {
                content: SpanData {
                    span: Span::from_parts("test", "0", 5..6),
                    data: "0".to_string(),
                },
            })),
        });

        let mut iter = input_tokens.into_iter().peekable();
        let result = Expression::parse(&mut iter);

        assert_eq!(None, iter.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn empty_function_call() {
        let input_content = "test()";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Expression::SingleOperation {
            base: Box::new(Expression::Identifier {
                ident: Identifier(SpanData {
                    span: Span::from_parts("test", "test", 0..4),
                    data: "test".to_string(),
                }),
            }),
            operation: SingleOperation::FuntionCall(vec![]),
        });

        let mut iter = input_tokens.into_iter().peekable();
        let result = Expression::parse(&mut iter);

        assert_eq!(None, iter.next());
        assert_eq!(expected, result);
    }
    #[test]
    fn one_arg_function_call() {
        let input_content = "test(0)";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Expression::SingleOperation {
            base: Box::new(Expression::Identifier {
                ident: Identifier(SpanData {
                    span: Span::from_parts("test", "test", 0..4),
                    data: "test".to_string(),
                }),
            }),
            operation: SingleOperation::FuntionCall(vec![Expression::Literal {
                content: SpanData {
                    span: Span::from_parts("test", "0", 5..6),
                    data: "0".to_string(),
                },
            }]),
        });

        let mut iter = input_tokens.into_iter().peekable();
        let result = Expression::parse(&mut iter);

        assert_eq!(None, iter.next());
        assert_eq!(expected, result);
    }
    #[test]
    fn two_args_function_call() {
        let input_content = "test(0,1)";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Expression::SingleOperation {
            base: Box::new(Expression::Identifier {
                ident: Identifier(SpanData {
                    span: Span::from_parts("test", "test", 0..4),
                    data: "test".to_string(),
                }),
            }),
            operation: SingleOperation::FuntionCall(vec![
                Expression::Literal {
                    content: SpanData {
                        span: Span::from_parts("test", "0", 5..6),
                        data: "0".to_string(),
                    },
                },
                Expression::Literal {
                    content: SpanData {
                        span: Span::from_parts("test", "1", 7..8),
                        data: "1".to_string(),
                    },
                },
            ]),
        });

        let mut iter = input_tokens.into_iter().peekable();
        let result = Expression::parse(&mut iter);

        assert_eq!(None, iter.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn array_literal_empty() {
        let input_content = "{}";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(Expression::ArrayLiteral { parts: Vec::new() });

        let mut iter = input_tokens.into_iter().peekable();
        let result = Expression::parse(&mut iter);

        assert_eq!(None, iter.next());
        assert_eq!(expected, result);
    }
}
