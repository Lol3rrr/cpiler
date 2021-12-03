// Operators in C:
// * https://www.tutorialspoint.com/cprogramming/c_operators_precedence.htm
// * https://en.cppreference.com/w/c/language/operator_precedence

use std::sync::Arc;

use general::{Source, Span, SpanData};
use itertools::PeekNth;
use tokenizer::{Assignment, Keyword, Operator, Token, TokenData};

use crate::{ExpectedToken, Identifier, SyntaxError, TypeToken};

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
    CharLiteral {
        content: SpanData<char>,
    },
    ArrayLiteral {
        parts: SpanData<Vec<Expression>>,
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
    SizeOf {
        ty: TypeToken,
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
    // TODO
    pub fn entire_span(&self) -> Option<Span> {
        match &self {
            Self::Identifier { ident } => Some(ident.0.span.clone()),
            Self::Literal { content } => Some(content.span.clone()),
            Self::StringLiteral { content } => Some(content.span.clone()),
            Self::CharLiteral { content } => Some(content.span.clone()),
            Self::ArrayLiteral { parts } => Some(parts.span.clone()),
            _ => None,
        }
    }

    // TODO
    pub fn sub_spans(&self) -> (Option<Span>, Option<Span>) {
        (None, None)
    }

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
            TokenData::CharLiteral { content } => {
                let mut chars = content.chars();
                let first_char = chars.next().ok_or(SyntaxError::UnexpectedEOF)?;

                let value = match first_char {
                    '\\' => {
                        let next = chars.next().ok_or(SyntaxError::UnexpectedEOF)?;

                        match next {
                            '0' => '\0',
                            'n' => '\n',
                            other => panic!("Unexpected Escape Sequence: {:?}", other),
                        }
                    }
                    other => other,
                };

                match chars.next() {
                    Some(_) => return Err(SyntaxError::UnexpectedEOF),
                    _ => {}
                };

                Ok(Self::CharLiteral {
                    content: SpanData {
                        span: current.span,
                        data: value,
                    },
                })
            }
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

            match (&current.data, state.get_cloned_last_token_data()) {
                (TokenData::Literal { .. }, Some(TokenData::Operator(_)))
                | (TokenData::Literal { .. }, None) => {
                    let entry = Self::parse_single_token(current)?;

                    state.add_expression(entry);
                }
                (TokenData::Literal { .. }, _) => {
                    return Err(SyntaxError::UnexpectedToken {
                        got: current.span,
                        expected: Some(vec![ExpectedToken::Identifier, ExpectedToken::Semicolon]),
                    });
                }
                (TokenData::StringLiteral { .. }, _) => {
                    let entry = Self::parse_single_token(current)?;

                    state.add_expression(entry);
                }
                (TokenData::CharLiteral { .. }, _) => {
                    let entry = Self::parse_single_token(current)?;

                    state.add_expression(entry);
                }
                (TokenData::Keyword(Keyword::SizeOf), _) => {
                    let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;
                    match &peeked.data {
                        TokenData::OpenParen => {
                            let _ = tokens.next();
                        }
                        _ => {}
                    };

                    let ty = TypeToken::parse(tokens)?;
                    dbg!(&ty);

                    if let Some(after_peeked) = tokens.peek() {
                        match &after_peeked.data {
                            TokenData::CloseParen => {
                                let _ = tokens.next();
                            }
                            _ => {}
                        };
                    }

                    let inner = Expression::SizeOf { ty };
                    state.add_expression(inner);
                }
                (TokenData::Operator(op), _) => {
                    match op {
                        Operator::BitwiseAnd => {
                            match state.get_cloned_last_token_data() {
                                Some(TokenData::Operator(_)) | None => {
                                    state.add_single_op(SingleOperation::AddressOf, current.span);
                                }
                                _ => {
                                    state.add_operator(&Operator::BitwiseAnd, current.span);
                                }
                            };
                        }
                        other => {
                            state.add_operator(other, current.span);
                        }
                    };
                }
                (TokenData::OpenParen, _) => {
                    match state.get_cloned_last_token_data() {
                        Some(TokenData::Literal { .. }) => {
                            dbg!("Got Function Call");

                            let params = Self::parse_exp_list(tokens, TokenData::CloseParen)?;

                            let closing_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                            match closing_token.data {
                                TokenData::CloseParen => {}
                                other => panic!("Expected ')' but got '{:?}'", other),
                            };

                            let start_range = current.span.source_area().start;
                            let end_range = closing_token.span.source_area().end;
                            let n_range = start_range..end_range;

                            let source = current.span.source();
                            let span = Span::new_arc_source(source.clone(), n_range);

                            state.add_single_op(SingleOperation::FuntionCall(params), span);
                        }
                        _ => {
                            let following_tok = {
                                let mut current_pos = 0;
                                let mut level = 0;
                                while let Some(tmp) = tokens.peek_nth(current_pos) {
                                    match &tmp.data {
                                        TokenData::CloseParen if level == 0 => break,
                                        TokenData::OpenParen => {
                                            level += 1;
                                        }
                                        TokenData::CloseParen => {
                                            level -= 1;
                                        }
                                        _ => {}
                                    };

                                    current_pos += 1;
                                }

                                let following_index = current_pos + 1;
                                tokens.peek_nth(following_index)
                            };

                            match &following_tok {
                                Some(tok) => {
                                    match &tok.data {
                                        TokenData::Operator(_)
                                        | TokenData::Semicolon
                                        | TokenData::Comma
                                        | TokenData::CloseParen => {
                                            let exp = Self::parse(tokens)?;

                                            let closing_token =
                                                tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                                            match closing_token.data {
                                                TokenData::CloseParen => {}
                                                _ => {
                                                    return Err(SyntaxError::UnexpectedToken {
                                                        expected: Some(vec![
                                                            ExpectedToken::CloseParen,
                                                        ]),
                                                        got: closing_token.span,
                                                    })
                                                }
                                            };

                                            state.add_expression(exp);
                                        }
                                        _ => {
                                            let target_ty = TypeToken::parse(tokens)?;

                                            let close_paren_token =
                                                tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                                            match close_paren_token.data {
                                                TokenData::CloseParen => {}
                                                _ => {
                                                    return Err(SyntaxError::UnexpectedToken {
                                                        got: close_paren_token.span,
                                                        expected: Some(vec![
                                                            ExpectedToken::CloseParen,
                                                        ]),
                                                    })
                                                }
                                            };

                                            let exp = Self::parse(tokens)?;

                                            let cast_exp = Self::Cast {
                                                target_ty,
                                                exp: Box::new(exp),
                                            };

                                            state.add_expression(cast_exp);
                                        }
                                    };
                                }
                                None => {
                                    let exp = Self::parse(tokens)?;

                                    let closing_token =
                                        tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                                    match closing_token.data {
                                        TokenData::CloseParen => {}
                                        _ => {
                                            return Err(SyntaxError::UnexpectedToken {
                                                expected: Some(vec![ExpectedToken::CloseParen]),
                                                got: closing_token.span,
                                            })
                                        }
                                    };

                                    state.add_expression(exp);
                                }
                            };
                        }
                    };
                }
                (TokenData::OpenBracket, _) => {
                    let exp = Self::parse(tokens)?;

                    let ending_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                    match ending_token.data {
                        TokenData::CloseBracket => {}
                        other => panic!("Expected ']' but got '{:?}'", other),
                    };

                    let start = current.span.source_area().start;
                    let end = ending_token.span.source_area().end;
                    let n_range = start..end;

                    let source: Arc<Source> = current.span.source().clone();
                    let span = Span::new_arc_source(source, n_range);

                    state.add_single_op(SingleOperation::ArrayAccess(Box::new(exp)), span);
                }
                (TokenData::OpenBrace, _) => {
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
                                    expected: Some(vec![
                                        ExpectedToken::Comma,
                                        ExpectedToken::CloseBrace,
                                    ]),
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
                                expected: Some(vec![ExpectedToken::CloseBrace]),
                                got: closing_token.span,
                            })
                        }
                    };

                    let entire_range =
                        current.span.source_area().start..closing_token.span.source_area().end;
                    let entire_span =
                        Span::new_arc_source(current.span.source().clone(), entire_range);
                    state.add_expression(Expression::ArrayLiteral {
                        parts: SpanData {
                            span: entire_span,
                            data: items,
                        },
                    });
                }
                (TokenData::QuestionMark, _) => {
                    let first = Self::parse(tokens)?;
                    dbg!(&first);

                    let seperator_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                    match seperator_token.data {
                        TokenData::Colon => {}
                        _ => {
                            return Err(SyntaxError::UnexpectedToken {
                                expected: Some(vec![ExpectedToken::Colon]),
                                got: seperator_token.span,
                            })
                        }
                    };

                    let second = Self::parse(tokens)?;
                    dbg!(&second);

                    let start = current.span.source_area().start;
                    let end = start;
                    let n_range = start..end;

                    let source: Arc<Source> = current.span.source().clone();
                    let span = Span::new_arc_source(source, n_range);

                    state.add_conditional(span);
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

        let result = state.finalize()?;
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
    use tokenizer::DataType;

    use crate::ExpressionReason;

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
    fn add_with_missing_operand() {
        let input_content = "1 + ";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Err(SyntaxError::ExpectedExpression {
            span: Span::new_source(source.clone(), 2..3),
            reason: ExpressionReason::Operand,
        });

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
                span: Span::new_source(source.clone(), 0..5),
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

        let expected = Ok(Expression::ArrayLiteral {
            parts: SpanData {
                span: Span::new_source(source.clone(), 0..2),
                data: Vec::new(),
            },
        });

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

    #[test]
    fn missing_operand_binary_op() {
        let input_content = "2 + ";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Err(SyntaxError::ExpectedExpression {
            span: Span::new_source(source.clone(), 2..3),
            reason: ExpressionReason::Operand,
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(expected, result);
    }
    #[test]
    fn missing_operand_unary_op() {
        let input_content = "!";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Err(SyntaxError::ExpectedExpression {
            span: Span::new_source(source.clone(), 0..1),
            reason: ExpressionReason::Operand,
        });

        let result = Expression::parse(&mut input_tokens);

        assert_eq!(expected, result);
    }

    #[test]
    fn entire_span() {
        let input_content = "123456";
        let input_source = Source::new("test", input_content);

        // All the Standalone ones that dont need anything else
        assert_eq!(
            Some(Span::new_source(input_source.clone(), 0..1)),
            Expression::Literal {
                content: SpanData {
                    span: Span::new_source(input_source.clone(), 0..1),
                    data: "1".to_string()
                }
            }
            .entire_span()
        );
        assert_eq!(
            Some(Span::new_source(input_source.clone(), 0..1)),
            Expression::StringLiteral {
                content: SpanData {
                    span: Span::new_source(input_source.clone(), 0..1),
                    data: "1".to_string(),
                }
            }
            .entire_span()
        );
        assert_eq!(
            Some(Span::new_source(input_source.clone(), 0..1)),
            Expression::CharLiteral {
                content: SpanData {
                    span: Span::new_source(input_source.clone(), 0..1),
                    data: '1',
                }
            }
            .entire_span()
        );

        // Array Literals are sort of special
        assert_eq!(
            Some(Span::new_source(input_source.clone(), 0..3)),
            Expression::ArrayLiteral {
                parts: SpanData {
                    span: Span::new_source(input_source.clone(), 0..3),
                    data: vec![
                        Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(input_source.clone(), 0..1),
                                data: "1".to_string()
                            }
                        },
                        Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(input_source.clone(), 1..2),
                                data: "2".to_string()
                            }
                        },
                        Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(input_source.clone(), 2..3),
                                data: "3".to_string()
                            }
                        }
                    ]
                },
            }
            .entire_span()
        );

        // The ones that are mainly alone
    }

    #[test]
    fn size_without_paren_simple() {
        let input = "sizeof int";
        let source = Source::new("test", input);
        let tokens = tokenizer::tokenize(source.clone().into());

        let expected = Ok(Expression::SizeOf {
            ty: TypeToken::Primitive(SpanData {
                span: Span::new_source(source.clone(), 7..10),
                data: DataType::Int,
            }),
        });

        let mut iter = peek_nth(tokens);
        let result = Expression::parse(&mut iter);

        assert_eq!(None, iter.next());
        assert_eq!(expected, result);
    }
    #[test]
    fn size_with_paren_simple() {
        let input = "sizeof(int)";
        let source = Source::new("test", input);
        let tokens = tokenizer::tokenize(source.clone().into());

        let expected = Ok(Expression::SizeOf {
            ty: TypeToken::Primitive(SpanData {
                span: Span::new_source(source.clone(), 7..10),
                data: DataType::Int,
            }),
        });

        let mut iter = peek_nth(tokens);
        let result = Expression::parse(&mut iter);

        assert_eq!(None, iter.next());
        assert_eq!(expected, result);
    }
}
