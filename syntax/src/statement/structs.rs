use std::iter::Peekable;

use itertools::PeekNth;
use tokenizer::{Token, TokenData};

use crate::{Identifier, SyntaxError, TypeToken};

#[derive(Debug, PartialEq)]
pub struct StructMembers {
    pub members: Vec<(TypeToken, Identifier)>,
}

impl StructMembers {
    pub fn parse<I>(tokens: &mut PeekNth<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let opening_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
        match opening_token.data {
            TokenData::OpenBrace => {}
            _ => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: Some(vec!["{".to_string()]),
                    got: opening_token.span,
                })
            }
        };

        let mut members = Vec::new();
        while let Some(peeked_tok) = tokens.peek() {
            dbg!(&peeked_tok);
            match &peeked_tok.data {
                TokenData::CloseBrace => {
                    let _ = tokens.next();
                    break;
                }
                _ => {}
            };

            let (ty, name) = TypeToken::parse_type_identifier(tokens)?;

            let semicolon_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
            match semicolon_tok.data {
                TokenData::Semicolon => {}
                _ => {
                    return Err(SyntaxError::UnexpectedToken {
                        expected: Some(vec![";".to_string()]),
                        got: semicolon_tok.span,
                    })
                }
            };

            members.push((ty, name));
        }

        Ok(Self { members })
    }
}

#[cfg(test)]
mod tests {
    use general::{Span, SpanData};
    use itertools::peek_nth;
    use tokenizer::DataType;

    use super::*;

    #[test]
    fn empty_struct_def() {
        let input = "{}";
        let input_span = Span::from_parts("test", input, 0..input.len());
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(StructMembers {
            members: Vec::new(),
        });

        let result = StructMembers::parse(&mut input_tokens);

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn one_item_def() {
        let input = "
        {
            int first;
        }";
        let input_span = Span::from_parts("test", input, 0..input.len());
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(StructMembers {
            members: vec![(
                TypeToken::Primitive(SpanData {
                    span: Span::from_parts("test", "int", 23..26),
                    data: DataType::Int,
                }),
                Identifier(SpanData {
                    span: Span::from_parts("test", "first", 27..32),
                    data: "first".to_string(),
                }),
            )],
        });

        let result = StructMembers::parse(&mut input_tokens);

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn two_item_def() {
        let input = "
        {
            int first;
            int second;
        }";
        let input_span = Span::from_parts("test", input, 0..input.len());
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(StructMembers {
            members: vec![
                (
                    TypeToken::Primitive(SpanData {
                        span: Span::from_parts("test", "int", 23..26),
                        data: DataType::Int,
                    }),
                    Identifier(SpanData {
                        span: Span::from_parts("test", "first", 27..32),
                        data: "first".to_string(),
                    }),
                ),
                (
                    TypeToken::Primitive(SpanData {
                        span: Span::from_parts("test", "int", 46..49),
                        data: DataType::Int,
                    }),
                    Identifier(SpanData {
                        span: Span::from_parts("test", "second", 50..56),
                        data: "second".to_string(),
                    }),
                ),
            ],
        });

        let result = StructMembers::parse(&mut input_tokens);

        assert_eq!(None, input_tokens.next());
        assert_eq!(expected, result);
    }
}
