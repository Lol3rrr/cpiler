use general::{shunting_yard, Span};

use crate::directive::ConditionalDirective;

use super::DefineManager;

mod iter;
pub use iter::InnerConditionalIterator;

#[derive(Debug, PartialEq)]
pub enum Conditional {
    Name {
        name: String,
    },
    Literal {
        value: String,
    },
    BinaryOp {
        left: Box<Self>,
        right: Box<Self>,
        op: ConditionalBinaryOp,
    },
    UnaryOp {
        base: Box<Self>,
        op: ConditionalUnaryOp,
    },
}

#[derive(Debug, PartialEq)]
pub enum ConditionalBinaryOp {
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub enum ConditionalUnaryOp {
    Not,
    Defined,
}

#[derive(Debug, PartialEq)]
enum ConditionalOp {
    Unary(ConditionalUnaryOp),
    Binary(ConditionalBinaryOp),
}

#[derive(Debug, PartialEq)]
enum Part {
    Name(String),
    Value(String),
    Operator(ConditionalOp),
    OpenParen,
    ClosingParen,
}

fn to_parts<'o, 's>(raw: &'s Span) -> impl Iterator<Item = Part> + 'o
where
    's: 'o,
{
    let mut char_in_iter = raw.content().char_indices().peekable();
    let mut last_end = 0;

    let mut raw_parts = Vec::new();

    while let Some((i, c)) = char_in_iter.next() {
        let peeked = char_in_iter.peek().map(|(_, c)| c);
        match (c, peeked) {
            (' ', _) => {
                if let Some(s) = raw.sub_span(last_end..i) {
                    raw_parts.push(s);
                }

                last_end = i + 1;
            }
            ('<', Some('='))
            | ('>', Some('='))
            | ('!', Some('='))
            | ('=', Some('='))
            | ('&', Some('&'))
            | ('|', Some('|')) => {
                if let Some(s) = raw.sub_span(last_end..i) {
                    raw_parts.push(s);
                }

                last_end = i;
                let _ = char_in_iter.next();

                let next_index = match char_in_iter.peek() {
                    Some((ti, _)) => *ti,
                    None => raw.content().len(),
                };

                if let Some(s) = raw.sub_span(last_end..next_index) {
                    raw_parts.push(s);
                }

                last_end = next_index;
            }
            ('<', _) | ('>', _) | ('!', _) | ('(', _) | (')', _) => {
                if let Some(s) = raw.sub_span(last_end..i) {
                    raw_parts.push(s);
                }

                last_end = i;

                if let Some(s) = raw.sub_span(last_end..i + 1) {
                    raw_parts.push(s);
                }

                last_end = i + 1;
            }
            _ => {}
        };
    }

    if let Some(s) = raw.sub_span(last_end..raw.content().len()) {
        raw_parts.push(s);
    }

    raw_parts
        .into_iter()
        .filter(|s| !s.content().is_empty())
        .map(|s| match s.content() {
            "<" => Part::Operator(ConditionalOp::Binary(ConditionalBinaryOp::Less)),
            ">" => Part::Operator(ConditionalOp::Binary(ConditionalBinaryOp::Greater)),
            "<=" => Part::Operator(ConditionalOp::Binary(ConditionalBinaryOp::LessEqual)),
            ">=" => Part::Operator(ConditionalOp::Binary(ConditionalBinaryOp::GreaterEqual)),
            "==" => Part::Operator(ConditionalOp::Binary(ConditionalBinaryOp::Equal)),
            "!=" => Part::Operator(ConditionalOp::Binary(ConditionalBinaryOp::NotEqual)),
            "!" => Part::Operator(ConditionalOp::Unary(ConditionalUnaryOp::Not)),
            "&&" => Part::Operator(ConditionalOp::Binary(ConditionalBinaryOp::And)),
            "||" => Part::Operator(ConditionalOp::Binary(ConditionalBinaryOp::Or)),
            "(" => Part::OpenParen,
            ")" => Part::ClosingParen,
            "defined" => Part::Operator(ConditionalOp::Unary(ConditionalUnaryOp::Defined)),
            other => {
                if s.content().chars().find(|c| !c.is_digit(10)).is_some() {
                    Part::Name(other.to_string())
                } else {
                    Part::Value(other.to_string())
                }
            }
        })
}

impl shunting_yard::Operator for ConditionalOp {
    type Value = Conditional;

    fn assosication(&self) -> shunting_yard::OpAssosication {
        match self {
            Self::Unary(ConditionalUnaryOp::Not) | Self::Unary(ConditionalUnaryOp::Defined) => {
                shunting_yard::OpAssosication::Right
            }
            Self::Binary(ConditionalBinaryOp::Less)
            | Self::Binary(ConditionalBinaryOp::Greater)
            | Self::Binary(ConditionalBinaryOp::LessEqual)
            | Self::Binary(ConditionalBinaryOp::GreaterEqual)
            | Self::Binary(ConditionalBinaryOp::NotEqual)
            | Self::Binary(ConditionalBinaryOp::Equal)
            | Self::Binary(ConditionalBinaryOp::And)
            | Self::Binary(ConditionalBinaryOp::Or) => shunting_yard::OpAssosication::Left,
        }
    }

    fn priority(&self) -> usize {
        match self {
            Self::Unary(ConditionalUnaryOp::Not) | Self::Unary(ConditionalUnaryOp::Defined) => 14,
            Self::Binary(ConditionalBinaryOp::Less)
            | Self::Binary(ConditionalBinaryOp::Greater)
            | Self::Binary(ConditionalBinaryOp::LessEqual)
            | Self::Binary(ConditionalBinaryOp::GreaterEqual) => 10,
            Self::Binary(ConditionalBinaryOp::Equal)
            | Self::Binary(ConditionalBinaryOp::NotEqual) => 9,
            Self::Binary(ConditionalBinaryOp::And) => 5,
            Self::Binary(ConditionalBinaryOp::Or) => 4,
        }
    }

    fn operands(&self) -> usize {
        match self {
            Self::Unary(_) => 1,
            Self::Binary(_) => 2,
        }
    }

    fn to_value(self, mut operands: Vec<Self::Value>) -> Self::Value {
        match self {
            Self::Unary(op) => {
                let base = operands.pop().unwrap();

                Conditional::UnaryOp {
                    base: Box::new(base),
                    op,
                }
            }
            Self::Binary(op) => {
                let left = operands.pop().unwrap();
                let right = operands.pop().unwrap();

                Conditional::BinaryOp {
                    left: Box::new(left),
                    right: Box::new(right),
                    op,
                }
            }
        }
    }
}

pub fn parse_conditional(raw: Span) -> Option<Conditional> {
    let parts = to_parts(&raw).map(|p| match p {
        Part::Name(n) => shunting_yard::Inputs::Value(Conditional::Name { name: n }),
        Part::Value(v) => shunting_yard::Inputs::Value(Conditional::Literal { value: v }),
        Part::Operator(op) => shunting_yard::Inputs::Op(shunting_yard::OpItem::Op(op)),
        Part::OpenParen => shunting_yard::Inputs::Op(shunting_yard::OpItem::OpenParen),
        Part::ClosingParen => shunting_yard::Inputs::Op(shunting_yard::OpItem::CloseParen),
    });

    shunting_yard::parse(parts).ok()
}

impl Conditional {
    fn intern_evaluate(self, defines: &DefineManager) -> Result<i64, ()> {
        match self {
            Self::Literal { value } => value.parse().map_err(|_| ()),
            Self::Name { name } => {
                dbg!(&name);
                todo!()
            }
            Self::UnaryOp { op, base } => match op {
                ConditionalUnaryOp::Not => {
                    let base_val = base.intern_evaluate(defines)?;

                    if base_val != 0 {
                        Ok(0)
                    } else {
                        Ok(1)
                    }
                }
                ConditionalUnaryOp::Defined => {
                    let name = match *base {
                        Conditional::Name { name } => name,
                        other => {
                            print!("Expected a Name but got {:?}", other);
                            return Err(());
                        }
                    };

                    let is_defined = defines.is_defined(&name);
                    if is_defined {
                        Ok(1)
                    } else {
                        Ok(0)
                    }
                }
            },
            Self::BinaryOp { op, left, right } => {
                let left_val = left.intern_evaluate(defines)?;
                let right_val = right.intern_evaluate(defines)?;

                match op {
                    ConditionalBinaryOp::Less => {
                        if left_val < right_val {
                            Ok(1)
                        } else {
                            Ok(0)
                        }
                    }
                    ConditionalBinaryOp::Greater => {
                        if left_val > right_val {
                            Ok(1)
                        } else {
                            Ok(0)
                        }
                    }
                    ConditionalBinaryOp::LessEqual => {
                        if left_val <= right_val {
                            Ok(1)
                        } else {
                            Ok(0)
                        }
                    }
                    ConditionalBinaryOp::GreaterEqual => {
                        if left_val < right_val {
                            Ok(1)
                        } else {
                            Ok(0)
                        }
                    }
                    ConditionalBinaryOp::Equal => {
                        if left_val == right_val {
                            Ok(1)
                        } else {
                            Ok(0)
                        }
                    }
                    ConditionalBinaryOp::NotEqual => {
                        if left_val != right_val {
                            Ok(1)
                        } else {
                            Ok(0)
                        }
                    }
                    ConditionalBinaryOp::Or => {
                        if left_val != 0 || right_val != 0 {
                            Ok(1)
                        } else {
                            Ok(0)
                        }
                    }
                    ConditionalBinaryOp::And => {
                        if left_val != 0 && right_val != 0 {
                            Ok(1)
                        } else {
                            Ok(0)
                        }
                    }
                }
            }
        }
    }

    pub fn evaluate(self, defines: &DefineManager) -> Result<bool, ()> {
        self.intern_evaluate(defines).map(|v| v != 0)
    }
}

impl TryFrom<ConditionalDirective> for Conditional {
    type Error = ();

    fn try_from(value: ConditionalDirective) -> Result<Self, Self::Error> {
        match value {
            ConditionalDirective::If { condition } => parse_conditional(condition).ok_or(()),
            ConditionalDirective::IfDef { name } => Ok(Self::UnaryOp {
                op: ConditionalUnaryOp::Defined,
                base: Box::new(Self::Name { name }),
            }),
            ConditionalDirective::IfNDef { name } => Ok(Self::UnaryOp {
                op: ConditionalUnaryOp::Not,
                base: Box::new(Self::UnaryOp {
                    op: ConditionalUnaryOp::Defined,
                    base: Box::new(Self::Name { name }),
                }),
            }),
            ConditionalDirective::Else => Ok(Self::Literal {
                value: "1".to_string(),
            }),
            ConditionalDirective::ElseIf { condition } => parse_conditional(condition).ok_or(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_condition_single() {
        let in_span = Span::new_source("test", "x < 0");

        let expected = Some(Conditional::BinaryOp {
            left: Box::new(Conditional::Name {
                name: "x".to_string(),
            }),
            right: Box::new(Conditional::Literal {
                value: "0".to_string(),
            }),
            op: ConditionalBinaryOp::Less,
        });

        let result = parse_conditional(in_span);

        assert_eq!(expected, result);
    }
}
