// Operators in C:
// * https://www.tutorialspoint.com/cprogramming/c_operators_precedence.htm
// * https://en.cppreference.com/w/c/language/operator_precedence

use std::iter::Peekable;

use general::SpanData;
use tokenizer::{Operator, Token, TokenData};

use crate::{Identifier, SyntaxError, TypeToken};

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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
}

#[derive(Debug, PartialEq)]
pub enum ExpressionOperator {
    Add,
    Sub,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug)]
enum Assosication {
    Left,
    Right,
}

#[derive(Debug)]
enum RpnOp {
    Expression(ExpressionOperator),
    SingleOp(SingleOperation),
}

impl RpnOp {
    fn precedence(&self) -> usize {
        match self {
            Self::Expression(ExpressionOperator::Add) => 2,
            Self::Expression(ExpressionOperator::Sub) => 2,
            Self::Expression(ExpressionOperator::Multiply) => 3,
            Self::Expression(ExpressionOperator::Divide) => 3,
            Self::Expression(ExpressionOperator::Modulo) => 3,
            Self::SingleOp(SingleOperation::Positive)
            | Self::SingleOp(SingleOperation::Negative)
            | Self::SingleOp(SingleOperation::LogicalNot)
            | Self::SingleOp(SingleOperation::BitwiseNot)
            | Self::SingleOp(SingleOperation::PrefixIncrement)
            | Self::SingleOp(SingleOperation::PrefixDecrement)
            | Self::SingleOp(SingleOperation::Cast(_))
            | Self::SingleOp(SingleOperation::Dereference)
            | Self::SingleOp(SingleOperation::AddressOf)
            | Self::SingleOp(SingleOperation::Sizeof) => 4,
            Self::SingleOp(SingleOperation::ArrayAccess(_))
            | Self::SingleOp(SingleOperation::Arrow)
            | Self::SingleOp(SingleOperation::Dot)
            | Self::SingleOp(SingleOperation::SuffixIncrement)
            | Self::SingleOp(SingleOperation::SuffixDecrement) => 5,
        }
    }

    fn assosication(&self) -> Assosication {
        match self {
            Self::Expression(ExpressionOperator::Add) => Assosication::Left,
            Self::Expression(ExpressionOperator::Sub) => Assosication::Left,
            Self::Expression(ExpressionOperator::Multiply) => Assosication::Left,
            Self::Expression(ExpressionOperator::Divide) => Assosication::Left,
            Self::Expression(ExpressionOperator::Modulo) => Assosication::Left,
            Self::SingleOp(SingleOperation::Positive)
            | Self::SingleOp(SingleOperation::Negative)
            | Self::SingleOp(SingleOperation::LogicalNot)
            | Self::SingleOp(SingleOperation::BitwiseNot)
            | Self::SingleOp(SingleOperation::PrefixIncrement)
            | Self::SingleOp(SingleOperation::PrefixDecrement)
            | Self::SingleOp(SingleOperation::Cast(_))
            | Self::SingleOp(SingleOperation::Dereference)
            | Self::SingleOp(SingleOperation::AddressOf)
            | Self::SingleOp(SingleOperation::Sizeof) => Assosication::Right,
            Self::SingleOp(SingleOperation::ArrayAccess(_))
            | Self::SingleOp(SingleOperation::Arrow)
            | Self::SingleOp(SingleOperation::Dot)
            | Self::SingleOp(SingleOperation::SuffixIncrement)
            | Self::SingleOp(SingleOperation::SuffixDecrement) => Assosication::Left,
        }
    }
}

#[derive(Debug)]
enum RPN {
    Expression(Expression),
    Operation(RpnOp),
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

    fn parse_expressions<I>(tokens: &mut Peekable<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        // This basically builds up the Expression using Reverse Polish Notation with the
        // shunting-yard-algorithm
        let mut output: Vec<RPN> = Vec::new();
        let mut op_stack: Vec<RpnOp> = Vec::new();
        let mut last_token_data: Option<TokenData> = None;

        while let Some(peeked) = tokens.peek() {
            match &peeked.data {
                TokenData::Semicolon => break,
                TokenData::CloseParen => break,
                TokenData::CloseBrace => break,
                _ => {}
            };

            let current = tokens.next().unwrap();
            let new_last_data = current.data.clone();

            match &current.data {
                TokenData::Literal { .. } => {
                    let entry = Self::parse_single_token(current)?;

                    output.push(RPN::Expression(entry));
                }
                TokenData::StringLiteral { .. } => {
                    let entry = Self::parse_single_token(current)?;

                    output.push(RPN::Expression(entry));
                }
                TokenData::Operator(op) => {
                    let exp_op = match (op, last_token_data) {
                        (Operator::Add, _) => RpnOp::Expression(ExpressionOperator::Add),
                        (Operator::Sub, Some(TokenData::Operator(_))) | (Operator::Sub, None) => {
                            RpnOp::SingleOp(SingleOperation::Negative)
                        }
                        (Operator::Sub, _) => RpnOp::Expression(ExpressionOperator::Sub),
                        (Operator::Multiply, _) => RpnOp::Expression(ExpressionOperator::Multiply),
                        (Operator::Divide, _) => RpnOp::Expression(ExpressionOperator::Divide),
                        (Operator::Modulo, _) => RpnOp::Expression(ExpressionOperator::Modulo),
                        (Operator::LogicalNot, _) => RpnOp::SingleOp(SingleOperation::LogicalNot),
                        (Operator::Dot, _) => {
                            todo!("Handle dot");
                        }
                        (op, _) => {
                            todo!("Handle unknown Op: {:?}", op);
                        }
                    };

                    let new_prec = exp_op.precedence();
                    let new_assoc = exp_op.assosication();

                    loop {
                        match op_stack.last() {
                            Some(latest) => {
                                let latest_prec = latest.precedence();

                                if new_prec < latest_prec {
                                    let popped = op_stack.pop().unwrap();
                                    output.push(RPN::Operation(popped));
                                    continue;
                                }

                                if new_prec == latest_prec {
                                    if let Assosication::Left = new_assoc {
                                        let popped = op_stack.pop().unwrap();
                                        output.push(RPN::Operation(popped));
                                        continue;
                                    }
                                }
                            }
                            None => {}
                        };

                        break;
                    }

                    op_stack.push(exp_op);
                }
                TokenData::OpenParen => {
                    let exp = Self::parse(tokens)?;

                    let peeked = tokens.peek();
                    dbg!(peeked);

                    output.push(RPN::Expression(exp));
                }
                TokenData::OpenBrace => {
                    let exp = Self::parse(tokens)?;

                    let peeked = tokens.peek();
                    dbg!(peeked);

                    output.push(RPN::Operation(RpnOp::SingleOp(
                        SingleOperation::ArrayAccess(Box::new(exp)),
                    )));
                }
                _ => {
                    return Err(SyntaxError::UnexpectedToken {
                        expected: None,
                        got: current.span,
                    })
                }
            };

            last_token_data = Some(new_last_data);
        }

        while let Some(op) = op_stack.pop() {
            output.push(RPN::Operation(op));
        }

        dbg!(&output);

        let mut final_stack: Vec<Expression> = Vec::new();
        for entry in output {
            match entry {
                RPN::Expression(exp) => {
                    final_stack.push(exp);
                }
                RPN::Operation(op) => {
                    match op {
                        RpnOp::SingleOp(op) => {
                            let base = final_stack.pop().unwrap();

                            let result = Expression::SingleOperation {
                                operation: op,
                                base: Box::new(base),
                            };

                            final_stack.push(result);
                        }
                        RpnOp::Expression(op) => {
                            let right = final_stack.pop().unwrap();
                            let left = final_stack.pop().unwrap();

                            let result = Expression::Operation {
                                left: Box::new(left),
                                operation: op,
                                right: Box::new(right),
                            };

                            final_stack.push(result);
                        }
                    };
                }
            };
        }

        let result = final_stack.pop().ok_or(SyntaxError::UnexpectedEOF)?;
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
}
