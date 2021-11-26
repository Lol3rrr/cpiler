use crate::{Span, SpanRef};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SpanData<D> {
    pub span: Span,
    pub data: D,
}

impl<D> SpanData<D>
where
    D: ParseSpan,
{
    pub fn parse<'s>(source: &SpanRef<'s>) -> Option<Self> {
        if source.content().trim().len() == 0 {
            return None;
        }

        let inner = D::parse(source)?;

        Some(Self {
            span: source.into(),
            data: inner,
        })
    }
}

pub trait ParseSpan
where
    Self: Sized,
{
    fn parse<'s>(source: &SpanRef<'s>) -> Option<Self>;
}
