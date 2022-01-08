use itertools::PeekNth;
use tokenizer::{Token, TokenData};

use crate::{EOFContext, ExpectedToken, Identifier, SyntaxError, TypeToken};

#[derive(Debug, PartialEq)]
pub struct StructMembers {
    pub members: Vec<(TypeToken, Identifier)>,
}

impl StructMembers {
    pub fn parse<I>(tokens: &mut PeekNth<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let opening_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
            ctx: EOFContext::Statement,
        })?;
        match opening_token.data {
            TokenData::OpenBrace => {}
            _ => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: Some(vec![ExpectedToken::OpenBrace]),
                    got: opening_token.span,
                })
            }
        };

        let mut members = Vec::new();
        while let Some(peeked_tok) = tokens.peek() {
            if peeked_tok.data == TokenData::CloseBrace {
                let _ = tokens.next();
                break;
            }

            let (ty, name) = TypeToken::parse_type_identifier(tokens)?;

            let semicolon_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                ctx: EOFContext::Statement,
            })?;
            match semicolon_tok.data {
                TokenData::Semicolon => {}
                _ => {
                    return Err(SyntaxError::UnexpectedToken {
                        expected: Some(vec![ExpectedToken::Semicolon]),
                        got: semicolon_tok.span,
                    })
                }
            };

            members.push((ty, name));
        }

        Ok(Self { members })
    }
}

impl IntoIterator for StructMembers {
    type IntoIter = std::vec::IntoIter<(TypeToken, Identifier)>;
    type Item = (TypeToken, Identifier);

    fn into_iter(self) -> Self::IntoIter {
        self.members.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use general::{Source, Span, SpanData};
    use itertools::peek_nth;
    use tokenizer::DataType;

    use super::*;

    #[test]
    fn empty_struct_def() {
        let input = "{}";
        let source = Source::new("test", input);
        let input_span: Span = source.clone().into();
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
        let source = Source::new("test", input);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(StructMembers {
            members: vec![(
                TypeToken::Primitive(SpanData {
                    span: Span::new_source(source.clone(), 23..26),
                    data: DataType::Int,
                }),
                Identifier(SpanData {
                    span: Span::new_source(source.clone(), 27..32),
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
        let source = Source::new("test", input);
        let input_span: Span = source.clone().into();
        let mut input_tokens = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(StructMembers {
            members: vec![
                (
                    TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 23..26),
                        data: DataType::Int,
                    }),
                    Identifier(SpanData {
                        span: Span::new_source(source.clone(), 27..32),
                        data: "first".to_string(),
                    }),
                ),
                (
                    TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 46..49),
                        data: DataType::Int,
                    }),
                    Identifier(SpanData {
                        span: Span::new_source(source.clone(), 50..56),
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
