use std::iter::Peekable;

use general::{shunting_yard, Span};

use crate::{directive::Directive, pir::PIR};

use super::DefineManager;

pub struct InnerConditionalIterator {
    inner: std::vec::IntoIter<PIR>,
}

impl InnerConditionalIterator {
    pub fn new<I>(base: &mut Peekable<I>) -> Self
    where
        I: Iterator<Item = PIR>,
    {
        let mut inner_part = Vec::new();

        let mut level = 0;
        while let Some(pir) = base.next() {
            match &pir {
                PIR::Directive((_, Directive::If { .. }))
                | PIR::Directive((_, Directive::IfDef { .. })) => {
                    level += 1;
                }
                PIR::Directive((_, Directive::Endif)) if level > 0 => {
                    level -= 1;
                }
                PIR::Directive((_, Directive::Else)) if level == 0 => {
                    while let Some(tmp) = base.next() {
                        match tmp {
                            PIR::Directive((_, Directive::Endif)) => break,
                            _ => {}
                        }
                    }
                }
                PIR::Directive((_, Directive::Endif)) if level == 0 => break,
                _ => {
                    inner_part.push(pir);
                }
            };
        }

        dbg!(&inner_part);

        Self {
            inner: inner_part.into_iter(),
        }
    }
}

impl Iterator for InnerConditionalIterator {
    type Item = PIR;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

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
                let (i, _) = char_in_iter.next().expect("We just peeked the next one");
                if let Some(s) = raw.sub_span(last_end..i) {
                    raw_parts.push(s);
                }

                last_end = i;

                if let Some(s) = raw.sub_span(last_end..i + 1) {
                    raw_parts.push(s);
                }

                last_end = i + 1;
            }
            ('<', _) | ('>', _) | ('!', _) => {
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
        Part::Operator(op) => shunting_yard::Inputs::Op(op),
    });

    shunting_yard::parse(parts).ok()
}

impl Conditional {
    fn intern_evaluate(self, defines: &DefineManager) -> Result<i64, ()> {
        dbg!(&self);

        match self {
            Self::Literal { value } => {
                dbg!(&value);
                todo!()
            }
            Self::Name { name } => {
                dbg!(&name);
                todo!()
            }
            Self::UnaryOp { op, base } => match op {
                ConditionalUnaryOp::Not => {
                    dbg!(&base);
                    todo!("Unary Not Operator");
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
                dbg!(&op, &left, &right);
                todo!()
            }
        }
    }

    pub fn evaluate(self, defines: &DefineManager) -> Result<bool, ()> {
        self.intern_evaluate(defines).map(|v| v != 0)
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
