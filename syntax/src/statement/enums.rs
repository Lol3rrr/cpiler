use general::SpanData;
use itertools::PeekNth;
use tokenizer::{Assignment, Token, TokenData};

use crate::{EOFContext, ExpectedToken, Identifier, SyntaxError};

#[derive(Debug, PartialEq, Clone)]
pub struct EnumVariant {
    pub name: Identifier,
    pub value: Option<SpanData<u64>>,
}

impl EnumVariant {
    pub fn parse<I>(tokens: &mut PeekNth<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let ident = Identifier::parse(tokens)?;

        let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
            ctx: EOFContext::Statement,
        })?;
        let value = match &peeked.data {
            TokenData::Assign(Assignment::Assign) => {
                let _ = tokens.next();

                let value_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Statement,
                })?;
                match value_token.data {
                    TokenData::Literal { content } => {
                        let i_value: u64 =
                            content.parse().map_err(|_| SyntaxError::UnexpectedToken {
                                got: value_token.span.clone(),
                                expected: Some(vec![ExpectedToken::IntegerLiteral]),
                            })?;

                        Some(SpanData {
                            span: value_token.span,
                            data: i_value,
                        })
                    }
                    _ => {
                        return Err(SyntaxError::UnexpectedToken {
                            got: value_token.span,
                            expected: Some(vec![ExpectedToken::Literal]),
                        })
                    }
                }
            }
            _ => None,
        };

        let next = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
            ctx: EOFContext::Statement,
        })?;
        match next.data {
            TokenData::Comma => {}
            _ => {
                return Err(SyntaxError::UnexpectedToken {
                    got: next.span,
                    expected: Some(vec![ExpectedToken::Comma]),
                })
            }
        };

        Ok(Self { name: ident, value })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumVariants {
    pub members: Vec<EnumVariant>,
}

impl EnumVariants {
    pub fn parse<I>(tokens: &mut PeekNth<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
            ctx: EOFContext::Statement,
        })?;
        match next_tok.data {
            TokenData::OpenBrace => {}
            _ => {
                return Err(SyntaxError::UnexpectedToken {
                    got: next_tok.span,
                    expected: Some(vec![ExpectedToken::OpenBrace]),
                })
            }
        };

        let mut members = Vec::new();
        loop {
            let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
                ctx: EOFContext::Statement,
            })?;
            if peeked.data == TokenData::CloseBrace {
                let _ = tokens.next();
                break;
            }

            let variant = EnumVariant::parse(tokens)?;
            members.push(variant);
        }

        Ok(Self { members })
    }
}
