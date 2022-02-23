use general::SpanData;
use itertools::PeekNth;
use tokenizer::{DataType, Keyword, Operator, Token, TokenData};

use crate::{EOFContext, ExpectedToken, Expression, Identifier, SyntaxError};

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum Modifier {
    Const,
    Signed,
    Unsigned,
    Long,
}

impl Modifier {
    pub fn is_modifier(token: &TokenData) -> bool {
        matches!(
            &token,
            TokenData::Keyword(Keyword::Const)
                | TokenData::Keyword(Keyword::DataType(DataType::Signed))
                | TokenData::Keyword(Keyword::DataType(DataType::Unsigned))
                | TokenData::Keyword(Keyword::DataType(DataType::Long))
        )
    }

    pub fn parse(data: TokenData) -> Option<Self> {
        match data {
            TokenData::Keyword(Keyword::Const) => Some(Self::Const),
            TokenData::Keyword(Keyword::DataType(DataType::Signed)) => Some(Self::Signed),
            TokenData::Keyword(Keyword::DataType(DataType::Unsigned)) => Some(Self::Unsigned),
            TokenData::Keyword(Keyword::DataType(DataType::Long)) => Some(Self::Long),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum TypeToken {
    /// A Pointer to some other Type
    Pointer(Box<Self>),
    /// A simple single Datatype, like an int or signed int
    Primitive(SpanData<DataType>),
    /// Handles types like "long long" or "unsigned int" and the like
    Composition {
        /// The "Modifier" that should be applied, like "unsigned" or "long"
        modifier: SpanData<Modifier>,
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
    fn parse_ty<I>(tokens: &mut PeekNth<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
            ctx: EOFContext::Type,
        })?;
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
                _ => return Ok(base),
            };
        }

        Ok(base)
    }

    pub fn parse<I>(tokens: &mut PeekNth<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
            ctx: EOFContext::Type,
        })?;
        match &peeked.data {
            TokenData::Keyword(Keyword::DataType(DataType::Short)) => {
                let next = tokens.next().unwrap();

                let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Type,
                })?;
                match &peeked.data {
                    TokenData::Literal { .. } => {}
                    TokenData::Keyword(Keyword::DataType(DataType::Int)) => {
                        let _ = tokens.next();
                    }
                    _ => {
                        let next = tokens.next().unwrap();

                        return Err(SyntaxError::UnexpectedToken {
                            got: next.span,
                            expected: None,
                        });
                    }
                };

                Ok(Self::Primitive(SpanData {
                    data: DataType::Short,
                    span: next.span,
                }))
            }
            TokenData::Keyword(Keyword::DataType(DataType::Long)) => {
                let next = tokens.next().unwrap();

                let peeked = tokens.peek().ok_or(SyntaxError::UnexpectedEOF {
                    ctx: EOFContext::Type,
                })?;
                match &peeked.data {
                    TokenData::Literal { .. } => Ok(Self::Primitive(SpanData {
                        data: DataType::Long,
                        span: next.span,
                    })),
                    TokenData::Keyword(Keyword::DataType(DataType::Int)) => {
                        let _ = tokens.next();

                        Ok(Self::Primitive(SpanData {
                            data: DataType::Long,
                            span: next.span,
                        }))
                    }
                    TokenData::Keyword(Keyword::DataType(DataType::Long)) => {
                        let base = Self::parse(tokens)?;

                        Ok(Self::Composition {
                            base: Box::new(base),
                            modifier: SpanData {
                                span: next.span,
                                data: Modifier::Long,
                            },
                        })
                    }
                    TokenData::Keyword(Keyword::DataType(DataType::Double)) => {
                        let base = Self::parse(tokens)?;

                        Ok(Self::Composition {
                            base: Box::new(base),
                            modifier: SpanData {
                                span: next.span,
                                data: Modifier::Long,
                            },
                        })
                    }
                    _ => {
                        let next = tokens.next().unwrap();

                        Err(SyntaxError::UnexpectedToken {
                            got: next.span,
                            expected: None,
                        })
                    }
                }
            }
            data if Modifier::is_modifier(data) => {
                let next = tokens.next().unwrap();

                let modif = Modifier::parse(next.data).unwrap();

                let base = Self::parse(tokens)?;

                Ok(Self::Composition {
                    base: Box::new(base),
                    modifier: SpanData {
                        span: next.span,
                        data: modif,
                    },
                })
            }
            _ => Self::parse_ty(tokens),
        }
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

        let ident = Identifier::parse(tokens)?;

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

            let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF {
                ctx: EOFContext::Type,
            })?;
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
            modifier: SpanData {
                span: Span::new_source(source.clone(), 0..8),
                data: Modifier::Unsigned,
            },
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
