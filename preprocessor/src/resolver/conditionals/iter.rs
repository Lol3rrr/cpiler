use std::iter::Peekable;

use crate::{
    directive::{ConditionalDirective, Directive},
    pir::PIR,
};

pub struct InnerConditionalIterator<'i> {
    level: usize,
    inner: Peekable<&'i mut dyn Iterator<Item = PIR>>,
}

impl<'i> InnerConditionalIterator<'i> {
    pub fn new<'b, I>(base: &'b mut I) -> Self
    where
        'b: 'i,
        I: Iterator<Item = PIR>,
    {
        let tmp = (base as &mut dyn Iterator<Item = PIR>).peekable();

        Self {
            level: 0,
            inner: tmp,
        }
    }
}

impl<'i> Iterator for InnerConditionalIterator<'i> {
    type Item = PIR;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let pir = self.inner.next()?;

            match &pir {
                PIR::Directive((_, Directive::Conditional(cond))) => {
                    dbg!(cond);
                    match cond {
                        ConditionalDirective::If { .. }
                        | ConditionalDirective::IfDef { .. }
                        | ConditionalDirective::IfNDef { .. }
                        | &ConditionalDirective::ElseIf { .. } => {
                            self.level += 1;
                        }
                        ConditionalDirective::Else if self.level == 0 => {
                            for tmp in self.inner.by_ref() {
                                if let PIR::Directive((_, Directive::EndIf)) = tmp {
                                    break;
                                }
                            }
                            continue;
                        }
                        ConditionalDirective::Else => {}
                    };
                }
                PIR::Directive((_, Directive::EndIf)) if self.level > 0 => {
                    self.level -= 1;
                }
                PIR::Directive((_, Directive::EndIf)) if self.level == 0 => return None,
                _ => {}
            };

            return Some(pir);
        }
    }
}

#[cfg(test)]
mod tests {
    use general::{Source, Span, SpanData};
    use tokenizer::{DataType, Keyword, TokenData};

    use crate::pir::into_pir;

    use super::*;

    #[test]
    fn inner_cond_iter_simple() {
        let input_content = "
#ifdef TMP
int first;
#else
int second;
#endif
            ";
        let source = Source::new("test", input_content);
        let input_toks = tokenizer::tokenize(source.clone().into());
        let mut input_pir = into_pir(input_toks).peekable();

        // We remove the First Element of it, because that is one of the assumptions we
        // make about the given input
        assert!(matches!(
            input_pir.next(),
            Some(PIR::Directive((
                _,
                Directive::Conditional(ConditionalDirective::IfDef { .. })
            )))
        ));

        let inner_iter = InnerConditionalIterator::new(&mut input_pir);

        let expected = vec![
            PIR::Token(SpanData {
                span: Span::new_source(source.clone(), 12..15),
                data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
            }),
            PIR::Token(SpanData {
                span: Span::new_source(source.clone(), 16..21),
                data: TokenData::Literal {
                    content: "first".to_string(),
                },
            }),
            PIR::Token(SpanData {
                span: Span::new_source(source, 21..22),
                data: TokenData::Semicolon,
            }),
        ];

        let result: Vec<_> = inner_iter.collect();
        dbg!(&result);

        // The PIR should now be empty, because the endif directive is the Element in the
        // given Input and should also be consumed
        assert_eq!(None, input_pir.next());

        assert_eq!(expected, result);
    }

    #[test]
    fn inner_cond_iter_nested() {
        let input_content = "
#ifdef TMP
#ifdef OTHER
int first;
#endif
#else
int second;
#endif
            ";
        let source = Source::new("test", input_content);
        let input_toks = tokenizer::tokenize(source.clone().into());
        let mut input_pir = into_pir(input_toks).peekable();

        // We remove the First Element of it, because that is one of the assumptions we
        // make about the given input
        assert!(matches!(
            input_pir.next(),
            Some(PIR::Directive((
                _,
                Directive::Conditional(ConditionalDirective::IfDef { .. })
            )))
        ));

        let inner_iter = InnerConditionalIterator::new(&mut input_pir);

        let expected = vec![
            PIR::Directive((
                SpanData {
                    span: Span::new_source(source.clone(), 13..24),
                    data: TokenData::CompilerDirective {
                        content: "ifdef OTHER".to_string(),
                    },
                },
                Directive::Conditional(ConditionalDirective::IfDef {
                    name: "OTHER".to_string(),
                }),
            )),
            PIR::Token(SpanData {
                span: Span::new_source(source.clone(), 25..28),
                data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
            }),
            PIR::Token(SpanData {
                span: Span::new_source(source.clone(), 29..34),
                data: TokenData::Literal {
                    content: "first".to_string(),
                },
            }),
            PIR::Token(SpanData {
                span: Span::new_source(source.clone(), 34..35),
                data: TokenData::Semicolon,
            }),
            PIR::Directive((
                SpanData {
                    span: Span::new_source(source, 37..42),
                    data: TokenData::CompilerDirective {
                        content: "endif".to_string(),
                    },
                },
                Directive::EndIf,
            )),
        ];

        let result: Vec<_> = inner_iter.collect();
        dbg!(&result);

        // The PIR should now be empty, because the endif directive is the Element in the
        // given Input and should also be consumed
        assert_eq!(None, input_pir.next());

        assert_eq!(expected, result);
    }
}
