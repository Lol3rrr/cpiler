use crate::{Span, SpanRef};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct SpanData<D> {
    pub span: Span,
    pub data: D,
}

impl<D> SpanData<D>
where
    D: ParseSpan,
{
    pub fn parse(source: &SpanRef<'_>) -> Option<Self> {
        if source.content().trim().is_empty() {
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
    fn parse(source: &SpanRef<'_>) -> Option<Self>;
}
