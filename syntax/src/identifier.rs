use general::{Span, SpanData};
use itertools::PeekNth;
use tokenizer::{Token, TokenData};

use crate::{ExpectedToken, SyntaxError};

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier(pub SpanData<String>);

impl Identifier {
    pub fn parse<I>(tokens: &mut PeekNth<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
        let name = match token.data {
            TokenData::Literal { content } => {
                // TODO
                // Verify the given content to be a valid identifier

                content
            }
            _ => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: Some(vec![ExpectedToken::Identifier]),
                    got: token.span,
                })
            }
        };

        Ok(Self(SpanData {
            span: token.span,
            data: name,
        }))
    }

    pub fn from_literal(span: Span, content: String) -> Result<Self, SyntaxError> {
        // TODO
        // Validate the Name
        match content.chars().next() {
            Some('0') | Some('1') | Some('2') | Some('3') | Some('4') | Some('5') | Some('6')
            | Some('7') | Some('8') | Some('9') | Some('_') | Some('-') | None => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: None,
                    got: span,
                })
            }
            _ => {}
        }

        Ok(Self(SpanData {
            span,
            data: content,
        }))
    }
}
