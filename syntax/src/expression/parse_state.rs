use general::{Span, SpanData};
use tokenizer::{Operator, TokenData};

use crate::{Expression, ExpressionOperator, ExpressionReason, SingleOperation, SyntaxError};

#[derive(Debug)]
enum Assosication {
    Left,
    Right,
}

#[derive(Debug)]
enum ConnectionOp {
    Dot,
    Arrow,
}

#[derive(Debug)]
enum RpnOp {
    Expression(ExpressionOperator),
    SingleOp(SingleOperation),
    ConnectionOp(ConnectionOp),
    Conditional,
}

impl RpnOp {
    fn precedence(&self) -> usize {
        match self {
            Self::SingleOp(SingleOperation::ArrayAccess(_))
            | Self::SingleOp(SingleOperation::FuntionCall(_))
            | Self::SingleOp(SingleOperation::Arrow)
            | Self::SingleOp(SingleOperation::Dot)
            | Self::SingleOp(SingleOperation::SuffixIncrement)
            | Self::SingleOp(SingleOperation::SuffixDecrement)
            | Self::ConnectionOp(ConnectionOp::Dot)
            | Self::ConnectionOp(ConnectionOp::Arrow) => 15,
            Self::SingleOp(SingleOperation::Positive)
            | Self::SingleOp(SingleOperation::Negative)
            | Self::SingleOp(SingleOperation::LogicalNot)
            | Self::SingleOp(SingleOperation::BitwiseNot)
            | Self::SingleOp(SingleOperation::PrefixIncrement)
            | Self::SingleOp(SingleOperation::PrefixDecrement)
            | Self::SingleOp(SingleOperation::Cast(_))
            | Self::SingleOp(SingleOperation::Dereference)
            | Self::SingleOp(SingleOperation::AddressOf)
            | Self::SingleOp(SingleOperation::Sizeof) => 14,
            Self::Expression(ExpressionOperator::Multiply)
            | Self::Expression(ExpressionOperator::Divide)
            | Self::Expression(ExpressionOperator::Modulo) => 13,
            Self::Expression(ExpressionOperator::Add)
            | Self::Expression(ExpressionOperator::Sub) => 12,
            Self::Expression(ExpressionOperator::ShiftLeft)
            | Self::Expression(ExpressionOperator::ShiftRight) => 11,
            Self::Expression(ExpressionOperator::Less)
            | Self::Expression(ExpressionOperator::Greater)
            | Self::Expression(ExpressionOperator::LessEqual)
            | Self::Expression(ExpressionOperator::GreaterEqual) => 10,
            Self::Expression(ExpressionOperator::Equal)
            | Self::Expression(ExpressionOperator::NotEqual) => 9,
            Self::Expression(ExpressionOperator::BitwiseAnd) => 8,
            Self::Expression(ExpressionOperator::BitwiseXor) => 7,
            Self::Expression(ExpressionOperator::BitwiseOr) => 6,
            Self::Expression(ExpressionOperator::LogicalAnd) => 5,
            Self::Expression(ExpressionOperator::LogicalOr) => 4,
            Self::Conditional => 3,
        }
    }

    fn assosication(&self) -> Assosication {
        match self {
            Self::Expression(ExpressionOperator::Add)
            | Self::Expression(ExpressionOperator::Sub)
            | Self::Expression(ExpressionOperator::Multiply)
            | Self::Expression(ExpressionOperator::Divide)
            | Self::Expression(ExpressionOperator::Modulo)
            | Self::Expression(ExpressionOperator::LogicalAnd)
            | Self::Expression(ExpressionOperator::LogicalOr)
            | Self::Expression(ExpressionOperator::BitwiseXor)
            | Self::Expression(ExpressionOperator::BitwiseAnd)
            | Self::Expression(ExpressionOperator::BitwiseOr)
            | Self::Expression(ExpressionOperator::ShiftLeft)
            | Self::Expression(ExpressionOperator::ShiftRight)
            | Self::Expression(ExpressionOperator::Equal)
            | Self::Expression(ExpressionOperator::NotEqual)
            | Self::Expression(ExpressionOperator::Less)
            | Self::Expression(ExpressionOperator::Greater)
            | Self::Expression(ExpressionOperator::GreaterEqual)
            | Self::Expression(ExpressionOperator::LessEqual) => Assosication::Left,
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
            | Self::SingleOp(SingleOperation::SuffixIncrement)
            | Self::SingleOp(SingleOperation::SuffixDecrement)
            | Self::SingleOp(SingleOperation::FuntionCall(_))
            | Self::SingleOp(SingleOperation::Arrow)
            | Self::SingleOp(SingleOperation::Dot)
            | Self::ConnectionOp(ConnectionOp::Dot)
            | Self::ConnectionOp(ConnectionOp::Arrow) => Assosication::Left,
            Self::Conditional => Assosication::Right,
        }
    }

    pub fn from_op(op: &Operator, previous_data: Option<TokenData>) -> Self {
        match (op, previous_data) {
            (Operator::Add, Some(TokenData::Operator(_))) | (Operator::Add, None) => {
                RpnOp::SingleOp(SingleOperation::Positive)
            }
            (Operator::Add, _) => RpnOp::Expression(ExpressionOperator::Add),
            (Operator::Increment, None) | (Operator::Increment, Some(TokenData::Operator(_))) => {
                RpnOp::SingleOp(SingleOperation::PrefixIncrement)
            }
            (Operator::Increment, Some(prev_tok)) => {
                dbg!(&prev_tok);

                RpnOp::SingleOp(SingleOperation::SuffixIncrement)
            }
            (Operator::Sub, Some(TokenData::Operator(_))) | (Operator::Sub, None) => {
                RpnOp::SingleOp(SingleOperation::Negative)
            }
            (Operator::Sub, _) => RpnOp::Expression(ExpressionOperator::Sub),
            (Operator::Decrement, None) | (Operator::Decrement, Some(TokenData::Operator(_))) => {
                RpnOp::SingleOp(SingleOperation::PrefixDecrement)
            }
            (Operator::Decrement, Some(prev_tok)) => {
                dbg!(&prev_tok);

                RpnOp::SingleOp(SingleOperation::SuffixDecrement)
            }
            (Operator::Multiply, _) => RpnOp::Expression(ExpressionOperator::Multiply),
            (Operator::Divide, _) => RpnOp::Expression(ExpressionOperator::Divide),
            (Operator::Modulo, _) => RpnOp::Expression(ExpressionOperator::Modulo),
            (Operator::LogicalNot, _) => RpnOp::SingleOp(SingleOperation::LogicalNot),
            (Operator::LogicalAnd, _) => RpnOp::Expression(ExpressionOperator::LogicalAnd),
            (Operator::LogicalOr, _) => RpnOp::Expression(ExpressionOperator::LogicalOr),
            (Operator::BitwiseNot, _) => RpnOp::SingleOp(SingleOperation::BitwiseNot),
            (Operator::BitwiseXor, _) => RpnOp::Expression(ExpressionOperator::BitwiseXor),
            (Operator::BitwiseOr, _) => RpnOp::Expression(ExpressionOperator::BitwiseOr),
            (Operator::BitwiseAnd, _) => RpnOp::Expression(ExpressionOperator::BitwiseAnd),
            (Operator::ShiftLeft, _) => RpnOp::Expression(ExpressionOperator::ShiftLeft),
            (Operator::ShiftRight, _) => RpnOp::Expression(ExpressionOperator::ShiftRight),
            (Operator::Equal, _) => RpnOp::Expression(ExpressionOperator::Equal),
            (Operator::NotEqual, _) => RpnOp::Expression(ExpressionOperator::NotEqual),
            (Operator::Less, _) => RpnOp::Expression(ExpressionOperator::Less),
            (Operator::Greater, _) => RpnOp::Expression(ExpressionOperator::Greater),
            (Operator::GreaterEqual, _) => RpnOp::Expression(ExpressionOperator::GreaterEqual),
            (Operator::LessEqual, _) => RpnOp::Expression(ExpressionOperator::LessEqual),
            (Operator::Arrow, _) => RpnOp::ConnectionOp(ConnectionOp::Arrow),
            (Operator::Dot, _) => RpnOp::ConnectionOp(ConnectionOp::Dot),
        }
    }
}

#[derive(Debug)]
enum RPN {
    Expression(Expression),
    Operation(RpnOp, Span),
}

pub struct ParseState {
    output: Vec<RPN>,
    op_stack: Vec<(RpnOp, Span)>,
    last_token_data: Option<TokenData>,
}

impl ParseState {
    pub fn new() -> Self {
        Self {
            output: Vec::new(),
            op_stack: Vec::new(),
            last_token_data: None,
        }
    }

    pub fn add_expression(&mut self, exp: Expression) {
        self.output.push(RPN::Expression(exp));
    }

    fn add_op(&mut self, op: RpnOp, span: Span) {
        let new_prec = op.precedence();
        let new_assoc = op.assosication();

        loop {
            if let Some((latest, _)) = self.op_stack.last() {
                let latest_prec = latest.precedence();

                if new_prec < latest_prec {
                    let (popped, p_span) = self.op_stack.pop().unwrap();
                    self.output.push(RPN::Operation(popped, p_span));
                    continue;
                }

                if new_prec == latest_prec {
                    if let Assosication::Left = new_assoc {
                        let (popped, p_span) = self.op_stack.pop().unwrap();
                        self.output.push(RPN::Operation(popped, p_span));
                        continue;
                    }
                }
            }

            break;
        }

        self.op_stack.push((op, span));
    }

    pub fn add_operator(&mut self, op: &Operator, span: Span) {
        let last_token_data = self.last_token_data.clone();

        let exp_op = RpnOp::from_op(op, last_token_data);
        self.add_op(exp_op, span);
    }

    pub fn add_single_op(&mut self, op: SingleOperation, span: Span) {
        self.add_op(RpnOp::SingleOp(op), span);
    }

    pub fn add_conditional(&mut self, span: Span) {
        self.add_op(RpnOp::Conditional, span);
    }

    pub fn get_cloned_last_token_data(&self) -> Option<TokenData> {
        self.last_token_data.clone()
    }
    pub fn set_last_token_data(&mut self, data: TokenData) {
        self.last_token_data = Some(data);
    }

    pub fn finalize(mut self) -> Result<Expression, SyntaxError> {
        while let Some((op, span)) = self.op_stack.pop() {
            self.output.push(RPN::Operation(op, span));
        }

        let mut final_stack: Vec<Expression> = Vec::new();
        for entry in self.output {
            match entry {
                RPN::Expression(exp) => {
                    final_stack.push(exp);
                }
                RPN::Operation(op, span) => {
                    match op {
                        RpnOp::SingleOp(op) => {
                            let base = final_stack.pop().ok_or_else(|| {
                                SyntaxError::ExpectedExpression {
                                    span: span.clone(),
                                    reason: ExpressionReason::Operand,
                                }
                            })?;

                            let result = Expression::SingleOperation {
                                operation: op,
                                base: Box::new(base),
                            };

                            final_stack.push(result);
                        }
                        RpnOp::Expression(op) => {
                            let right = final_stack.pop().ok_or_else(|| {
                                SyntaxError::ExpectedExpression {
                                    span: span.clone(),
                                    reason: ExpressionReason::Operand,
                                }
                            })?;
                            let left = final_stack.pop().ok_or_else(|| {
                                SyntaxError::ExpectedExpression {
                                    span: span.clone(),
                                    reason: ExpressionReason::Operand,
                                }
                            })?;

                            let result = Expression::Operation {
                                left: Box::new(left),
                                operation: op,
                                right: Box::new(right),
                            };

                            final_stack.push(result);
                        }
                        RpnOp::ConnectionOp(con_op) => {
                            let right = final_stack.pop().unwrap();
                            let left = final_stack.pop().unwrap();

                            let result = match (right, con_op) {
                                (Expression::Identifier { ident }, _) => Expression::StructAccess {
                                    base: Box::new(left),
                                    field: ident,
                                },
                                (right, ConnectionOp::Dot) => match (left, right) {
                                    (
                                        Expression::Literal {
                                            content: left_content,
                                        },
                                        Expression::Literal {
                                            content: right_content,
                                        },
                                    ) => {
                                        let left_str = &left_content.data;
                                        if left_str.chars().any(|c| !c.is_digit(10)) {
                                            todo!("Left side is not a number");
                                        }

                                        let right_span = {
                                            let tmp = &right_content.data;
                                            if tmp.ends_with('f') || tmp.ends_with('d') {
                                                right_content
                                                    .span
                                                    .sub_span(0..tmp.len() - 1)
                                                    .unwrap()
                                            } else {
                                                right_content.span.sub_span(0..tmp.len()).unwrap()
                                            }
                                        };

                                        if right_span.content().chars().any(|c| !c.is_digit(10)) {
                                            todo!("Right side is not a number");
                                        }

                                        let combined_span =
                                            left_content.span.join(right_span.into());

                                        Expression::Literal {
                                            content: SpanData {
                                                data: combined_span.content().to_owned(),
                                                span: combined_span,
                                            },
                                        }
                                    }
                                    (left, right) => {
                                        panic!("Unexpected Dot Combination: {:?}.{:?}", left, right)
                                    }
                                },
                                other => panic!("Expected Identifier but got: {:?}", other),
                            };

                            final_stack.push(result);
                        }
                        RpnOp::Conditional => {
                            let false_exp = final_stack.pop().unwrap();
                            let true_exp = final_stack.pop().unwrap();
                            let cond = final_stack.pop().unwrap();

                            let result = Expression::Conditional {
                                condition: Box::new(cond),
                                first: Box::new(true_exp),
                                second: Box::new(false_exp),
                            };

                            final_stack.push(result);
                        }
                    };
                }
            };
        }

        final_stack.pop().ok_or(SyntaxError::UnexpectedEOF)
    }
}
