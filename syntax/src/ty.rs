use general::SpanData;
use itertools::PeekNth;
use tokenizer::{DataType, Keyword, Operator, Token, TokenData};

use crate::{ExpectedToken, Expression, Identifier, SyntaxError};

#[derive(Debug, PartialEq, Clone)]
pub enum TypeToken {
    /// A Pointer to some other Type
    Pointer(Box<Self>),
    /// A simple single Datatype, like an int or signed int
    Primitive(SpanData<DataType>),
    /// Handles types like "long long" or "unsigned int" and the like
    Composition {
        /// The "Modifier" that should be applied, like "unsigned" or "long"
        modifier: Box<Self>,
        base: Box<Self>,
    },
    /// Datatypes that have been delcared using typedef
    TypeDefed {
        /// The Name of the Custom Type
        name: Identifier,
    },
    /// Struct Types
    StructType {
        /// The Name of the Struct
        name: Identifier,
    },
    /// Enum Types
    EnumType {
        /// The Name of the Enum
        name: Identifier,
    },
    /// An Array with constant Size
    ArrayType {
        base: Box<Self>,
        size: Option<Box<Expression>>,
    },
}

impl TypeToken {
    pub fn parse<I>(tokens: &mut PeekNth<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
        let mut base = match next_tok.data {
            TokenData::Keyword(Keyword::DataType(DataType::Struct)) => {
                let name = Identifier::parse(tokens)?;

                TypeToken::StructType { name }
            }
            TokenData::Keyword(Keyword::DataType(DataType::Enum)) => {
                let name = Identifier::parse(tokens)?;

                TypeToken::EnumType { name }
            }
            TokenData::Keyword(Keyword::DataType(dt)) => {
                let base = dt;

                Self::Primitive(SpanData {
                    span: next_tok.span,
                    data: base,
                })
            }
            TokenData::Literal { content } => {
                let name = Identifier::from_literal(next_tok.span, content)?;
                Self::TypeDefed { name }
            }
            _ => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: Some(vec![ExpectedToken::Identifier]),
                    got: next_tok.span,
                })
            }
        };

        while let Some(peeked) = tokens.peek() {
            match &peeked.data {
                TokenData::Operator(Operator::Multiply) => {
                    let _ = tokens.next();
                    base = Self::Pointer(Box::new(base));
                }
                TokenData::Keyword(Keyword::DataType(_)) => {
                    let mod_base = Self::parse(tokens)?;

                    base = Self::Composition {
                        modifier: Box::new(base),
                        base: Box::new(mod_base),
                    };
                }
                _ => return Ok(base),
            };
        }

        Ok(base)
    }

    /// This should be used to parse combinations of form "type identifier", as
    /// this will handle it correctly for you while also accounting for certain
    /// Problems like Arrays and the like
    pub fn parse_type_identifier<I>(
        tokens: &mut PeekNth<I>,
    ) -> Result<(Self, Identifier), SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let mut base = Self::parse(tokens)?;
        dbg!(&base);

        let ident = Identifier::parse(tokens)?;
        dbg!(&ident);

        loop {
            let peeked = match tokens.peek() {
                Some(p) => p,
                None => return Ok((base, ident)),
            };

            match &peeked.data {
                TokenData::OpenBracket => {
                    let _ = tokens.next();
                }
                _ => return Ok((base, ident)),
            };

            let size_exp = match tokens.peek() {
                Some(_) => {
                    let exp = Expression::parse(tokens)?;
                    Some(Box::new(exp))
                }
                None => None,
            };

            let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
            match next_tok.data {
                TokenData::CloseBracket => {}
                _ => {
                    return Err(SyntaxError::UnexpectedToken {
                        expected: None,
                        got: next_tok.span,
                    })
                }
            };

            base = Self::ArrayType {
                base: Box::new(base),
                size: size_exp,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use general::{Source, Span};
    use itertools::peek_nth;

    use super::*;

    #[test]
    fn primitive_type() {
        let input_content = "int";
        let source = Source::new("test", input_content);

        let input_span: Span = source.clone().into();
        let mut tokenized = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(TypeToken::Primitive(SpanData {
            span: Span::new_source(source.clone(), 0..3),
            data: DataType::Int,
        }));

        let result = TypeToken::parse(&mut tokenized);

        assert_eq!(expected, result);
    }

    #[test]
    fn pointer() {
        let input_content = "int*";
        let source = Source::new("test", input_content);

        let input_span: Span = source.clone().into();
        let mut tokenized = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(TypeToken::Pointer(Box::new(TypeToken::Primitive(
            SpanData {
                span: Span::new_source(source.clone(), 0..3),
                data: DataType::Int,
            },
        ))));

        let result = TypeToken::parse(&mut tokenized);

        assert_eq!(expected, result);
    }

    #[test]
    fn nested_pointer() {
        let input_content = "int**";
        let source = Source::new("test", input_content);

        let input_span: Span = source.clone().into();
        let mut tokenized = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(TypeToken::Pointer(Box::new(TypeToken::Pointer(Box::new(
            TypeToken::Primitive(SpanData {
                span: Span::new_source(source.clone(), 0..3),
                data: DataType::Int,
            }),
        )))));

        let result = TypeToken::parse(&mut tokenized);

        assert_eq!(expected, result);
    }

    #[test]
    fn modified_type() {
        let input_content = "unsigned int";
        let source = Source::new("test", input_content);

        let input_span: Span = source.clone().into();
        let mut tokenized = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(TypeToken::Composition {
            modifier: Box::new(TypeToken::Primitive(SpanData {
                span: Span::new_source(source.clone(), 0..8),
                data: DataType::Unsigned,
            })),
            base: Box::new(TypeToken::Primitive(SpanData {
                span: Span::new_source(source.clone(), 9..12),
                data: DataType::Int,
            })),
        });

        let result = TypeToken::parse(&mut tokenized);

        assert_eq!(expected, result);
    }

    #[test]
    fn simple_struct() {
        let input_content = "struct testing";
        let source = Source::new("test", input_content);

        let input_span: Span = source.clone().into();
        let mut tokenized = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok(TypeToken::StructType {
            name: Identifier(SpanData {
                span: Span::new_source(source.clone(), 7..14),
                data: "testing".to_string(),
            }),
        });

        let result = TypeToken::parse(&mut tokenized);

        assert_eq!(expected, result);
    }

    #[test]
    fn type_name_combination_primitive() {
        let input_content = "int testing;";
        let source = Source::new("test", input_content);

        let input_span: Span = source.clone().into();
        let mut tokenized = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok((
            TypeToken::Primitive(SpanData {
                span: Span::new_source(source.clone(), 0..3),
                data: DataType::Int,
            }),
            Identifier(SpanData {
                span: Span::new_source(source.clone(), 4..11),
                data: "testing".to_string(),
            }),
        ));

        let result = TypeToken::parse_type_identifier(&mut tokenized);

        assert!(tokenized.next().is_some());
        assert_eq!(None, tokenized.next());

        assert_eq!(expected, result);
    }
    #[test]
    fn type_name_combination_primitive_array_known_size() {
        let input_content = "int testing[13];";
        let source = Source::new("test", input_content);

        let input_span: Span = source.clone().into();
        let mut tokenized = peek_nth(tokenizer::tokenize(input_span));

        let expected = Ok((
            TypeToken::ArrayType {
                base: Box::new(TypeToken::Primitive(SpanData {
                    span: Span::new_source(source.clone(), 0..3),
                    data: DataType::Int,
                })),
                size: Some(Box::new(Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 12..14),
                        data: "13".to_string(),
                    },
                })),
            },
            Identifier(SpanData {
                span: Span::new_source(source.clone(), 4..11),
                data: "testing".to_string(),
            }),
        ));

        let result = TypeToken::parse_type_identifier(&mut tokenized);

        assert!(tokenized.next().is_some());
        assert_eq!(None, tokenized.next());

        assert_eq!(expected, result);
    }
}
