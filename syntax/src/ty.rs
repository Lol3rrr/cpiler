use std::iter::Peekable;

use general::SpanData;
use tokenizer::{DataType, Keyword, Operator, Token, TokenData};

use crate::{Expression, Identifier, SyntaxError};

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
    pub fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let next_tok = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;

        match next_tok.data {
            TokenData::Keyword(Keyword::DataType(DataType::Struct)) => {
                let name = Identifier::parse(tokens)?;

                Ok(TypeToken::StructType { name })
            }
            TokenData::Keyword(Keyword::DataType(DataType::Enum)) => {
                let name = Identifier::parse(tokens)?;

                Ok(TypeToken::EnumType { name })
            }
            TokenData::Keyword(Keyword::DataType(dt)) => {
                let base = dt;

                let mut base = Self::Primitive(SpanData {
                    span: next_tok.span,
                    data: base,
                });

                while let Some(peeked) = tokens.peek() {
                    dbg!(&peeked);
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
            _ => Err(SyntaxError::UnexpectedToken {
                expected: Some(vec!["Identifier".to_string()]),
                got: next_tok.span,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use general::Span;

    use super::*;

    #[test]
    fn primitive_type() {
        let input_content = "int";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let tokenized = tokenizer::tokenize(input_span);

        let expected = Ok(TypeToken::Primitive(SpanData {
            span: Span::from_parts("test", "int", 0..3),
            data: DataType::Int,
        }));

        let result = TypeToken::parse(&mut tokenized.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn pointer() {
        let input_content = "int*";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let tokenized = tokenizer::tokenize(input_span);

        let expected = Ok(TypeToken::Pointer(Box::new(TypeToken::Primitive(
            SpanData {
                span: Span::from_parts("test", "int", 0..3),
                data: DataType::Int,
            },
        ))));

        let result = TypeToken::parse(&mut tokenized.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn nested_pointer() {
        let input_content = "int**";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let tokenized = tokenizer::tokenize(input_span);

        let expected = Ok(TypeToken::Pointer(Box::new(TypeToken::Pointer(Box::new(
            TypeToken::Primitive(SpanData {
                span: Span::from_parts("test", "int", 0..3),
                data: DataType::Int,
            }),
        )))));

        let result = TypeToken::parse(&mut tokenized.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn modified_type() {
        let input_content = "unsigned int";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let tokenized = tokenizer::tokenize(input_span);

        let expected = Ok(TypeToken::Composition {
            modifier: Box::new(TypeToken::Primitive(SpanData {
                span: Span::from_parts("test", "unsigned", 0..8),
                data: DataType::Unsigned,
            })),
            base: Box::new(TypeToken::Primitive(SpanData {
                span: Span::from_parts("test", "int", 9..12),
                data: DataType::Int,
            })),
        });

        let result = TypeToken::parse(&mut tokenized.into_iter().peekable());

        assert_eq!(expected, result);
    }

    #[test]
    fn simple_struct() {
        let input_content = "struct testing";
        let input_span = Span::from_parts("test", input_content, 0..input_content.len());
        let tokenized = tokenizer::tokenize(input_span);

        let expected = Ok(TypeToken::StructType {
            name: Identifier(SpanData {
                span: Span::from_parts("test", "testing", 7..14),
                data: "testing".to_string(),
            }),
        });

        let result = TypeToken::parse(&mut tokenized.into_iter().peekable());

        assert_eq!(expected, result);
    }
}
