use itertools::PeekNth;
use tokenizer::{Operator, Token, TokenData};

use crate::{ExpectedToken, Expression, Identifier, SyntaxError};

#[derive(Debug, PartialEq)]
pub enum AssignTarget {
    Variable(Identifier),
    ArrayAccess { base: Box<Self>, index: Expression },
    StructAccess { base: Box<Self>, field: Identifier },
    StructPtrAccess { base: Box<Self>, field: Identifier },
}

impl AssignTarget {
    pub fn parse<I>(tokens: &mut PeekNth<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let ident = Identifier::parse(tokens)?;
        let mut base = Self::Variable(ident);

        while let Some(peeked_token) = tokens.peek() {
            match &peeked_token.data {
                TokenData::OpenBracket => {
                    let _ = tokens.next();

                    let index_exp = Expression::parse(tokens)?;

                    let close_token = tokens.next().ok_or(SyntaxError::UnexpectedEOF)?;
                    match close_token.data {
                        TokenData::CloseBracket => {}
                        _ => {
                            return Err(SyntaxError::UnexpectedToken {
                                expected: Some(vec![ExpectedToken::CloseBracket]),
                                got: close_token.span,
                            })
                        }
                    };

                    base = Self::ArrayAccess {
                        base: Box::new(base),
                        index: index_exp,
                    };
                }
                TokenData::Operator(Operator::Dot) => {
                    let _ = tokens.next();

                    let field_name = Identifier::parse(tokens)?;

                    base = Self::StructAccess {
                        base: Box::new(base),
                        field: field_name,
                    };
                }
                TokenData::Operator(Operator::Arrow) => {
                    let _ = tokens.next();

                    let field_name = Identifier::parse(tokens)?;

                    base = Self::StructPtrAccess {
                        base: Box::new(base),
                        field: field_name,
                    };
                }
                _ => break,
            };
        }

        Ok(base)
    }

    /// Converts the Target to an Expression that can be used to "load" this Target, which
    /// is mainly used when handling assignments with operators like "+="
    pub fn to_exp(&self) -> Expression {
        dbg!(&self);
        match self {
            Self::Variable(ident) => Expression::Identifier {
                ident: ident.to_owned(),
            },
            Self::StructAccess { base, field } => Expression::StructAccess {
                field: field.clone(),
                base: Box::new(base.to_exp()),
            },
            other => todo!("{:?}", other),
        }
    }
}

#[cfg(test)]
mod tests {
    use general::{Source, Span, SpanData};
    use itertools::peek_nth;

    use super::*;

    #[test]
    fn variable() {
        let input_content = "test";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(AssignTarget::Variable(Identifier(SpanData {
            span: Span::new_source(source.clone(), 0..4),
            data: "test".to_string(),
        })));

        let mut iter = peek_nth(input_tokens);
        let result = AssignTarget::parse(&mut iter);

        assert_eq!(None, iter.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn array_access() {
        let input_content = "test[0]";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(AssignTarget::ArrayAccess {
            base: Box::new(AssignTarget::Variable(Identifier(SpanData {
                span: Span::new_source(source.clone(), 0..4),
                data: "test".to_string(),
            }))),
            index: Expression::Literal {
                content: SpanData {
                    span: Span::new_source(source.clone(), 5..6),
                    data: "0".to_string(),
                },
            },
        });

        let mut iter = peek_nth(input_tokens);
        let result = AssignTarget::parse(&mut iter);

        assert_eq!(None, iter.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn struct_access() {
        let input_content = "test.field";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(AssignTarget::StructAccess {
            base: Box::new(AssignTarget::Variable(Identifier(SpanData {
                span: Span::new_source(source.clone(), 0..4),
                data: "test".to_string(),
            }))),
            field: Identifier(SpanData {
                span: Span::new_source(source.clone(), 5..10),
                data: "field".to_string(),
            }),
        });

        let mut iter = peek_nth(input_tokens);
        let result = AssignTarget::parse(&mut iter);

        assert_eq!(None, iter.next());
        assert_eq!(expected, result);
    }

    #[test]
    fn struct_ptr_access() {
        let input_content = "test->field";
        let source = Source::new("test", input_content);
        let input_span: Span = source.clone().into();
        let input_tokens = tokenizer::tokenize(input_span);

        let expected = Ok(AssignTarget::StructPtrAccess {
            base: Box::new(AssignTarget::Variable(Identifier(SpanData {
                span: Span::new_source(source.clone(), 0..4),
                data: "test".to_string(),
            }))),
            field: Identifier(SpanData {
                span: Span::new_source(source.clone(), 6..11),
                data: "field".to_string(),
            }),
        });

        let mut iter = peek_nth(input_tokens);
        let result = AssignTarget::parse(&mut iter);

        assert_eq!(None, iter.next());
        assert_eq!(expected, result);
    }
}
