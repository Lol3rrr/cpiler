use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Range,
    sync::Arc,
};

// TODO
// Potentially switch the source String to an Arc<String> and then whenever
// you construct a subspan you dont clone the String itself but rather only the
// Arc which should be faster and more efficient

mod char_iter;
pub use char_iter::CharIndexIter;

use crate::Source;

/// A Span describes a Part of some overall String, most likely source Code
#[derive(PartialEq, Clone)]
pub struct Span {
    /// The Source Content (most likely a File)
    source: Arc<Source>,
    /// The Area in the Source which corresponds to the Content of this File
    source_area: Range<usize>,
}

impl Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.content().as_bytes());
    }
}

impl Eq for Span {}

impl Span {
    pub fn new_source(source: Source, range: Range<usize>) -> Self {
        Self {
            source: Arc::new(source),
            source_area: range,
        }
    }

    pub fn new_arc_source(source: Arc<Source>, range: Range<usize>) -> Self {
        Self {
            source,
            source_area: range,
        }
    }

    pub fn sub_span<'out>(&'out self, range: Range<usize>) -> Option<SpanRef<'out>> {
        let length = self.source_area.len();
        if range.start > length || range.end > length {
            return None;
        }

        let source_start = self.source_area.start;
        let sub_area = source_start + range.start..source_start + range.end;

        Some(SpanRef {
            source: &self.source,
            source_area: sub_area,
        })
    }

    pub fn content(&'_ self) -> &'_ str {
        self.source.sub_content(self.source_area.clone()).expect("")
    }
    pub fn source(&self) -> &Arc<Source> {
        &self.source
    }
    pub fn source_area(&self) -> &Range<usize> {
        &self.source_area
    }

    pub fn join<C>(self, other: Self, combinator: C) -> Self
    where
        C: Display,
    {
        let n_range = self.source_area.start..other.source_area.end;

        Self {
            source: self.source,
            source_area: n_range,
        }
    }
}

impl AsRef<str> for Span {
    fn as_ref(&self) -> &str {
        self.content()
    }
}
impl From<Source> for Span {
    fn from(source: Source) -> Self {
        let source_range = 0..source.content().len();

        Self::new_source(source, source_range)
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Span {{ source: {:?}, source_area: {:?}, content: {:?} }}",
            self.source(),
            self.source_area(),
            self.content()
        )
    }
}

#[derive(PartialEq, Clone)]
pub struct SpanRef<'a> {
    source: &'a Arc<Source>,
    source_area: Range<usize>,
}

impl<'a> Debug for SpanRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Span {{ source: {:?}, source_area: {:?}, content: {:?} }}",
            self.source,
            self.source_area,
            self.content()
        )
    }
}

impl<'a> SpanRef<'a> {
    pub fn content(&self) -> &str {
        self.source.sub_content(self.source_area.clone()).expect("")
    }

    pub fn sub_span<'out>(&'out self, range: Range<usize>) -> Option<SpanRef<'out>> {
        let length = self.source_area.len();
        if range.start > length || range.end > length {
            return None;
        }

        let source_start = self.source_area.start;
        let sub_area = source_start + range.start..source_start + range.end;

        Some(SpanRef {
            source: &self.source,
            source_area: sub_area,
        })
    }
}

impl<'s> Into<Span> for SpanRef<'s> {
    fn into(self) -> Span {
        Span {
            source: self.source.to_owned(),
            source_area: self.source_area,
        }
    }
}
impl<'s> Into<Span> for &SpanRef<'s> {
    fn into(self) -> Span {
        Span {
            source: self.source.to_owned(),
            source_area: self.source_area.clone(),
        }
    }
}
impl<'o, 's> From<&'o Span> for SpanRef<'s>
where
    'o: 's,
{
    fn from(source: &'o Span) -> Self {
        Self {
            source: &source.source,
            source_area: source.source_area.clone(),
        }
    }
}

impl<'a> AsRef<str> for SpanRef<'a> {
    fn as_ref(&self) -> &str {
        self.content()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_subspan_beginning() {
        let source = Source::new("testing", "abcdefghijklmonpqrstuvw");
        let arced_source = Arc::new(source.clone());

        let source_span: Span = source.into();
        let expected_sub = Some(SpanRef {
            source: &arced_source,
            source_area: 0..6,
        });

        let result = source_span.sub_span(0..6);

        assert_eq!(expected_sub, result);
    }

    #[test]
    fn valid_subspan_middle() {
        let source = Source::new("testing", "abcdefghijklmonpqrstuvw");
        let arced_source = Arc::new(source.clone());

        let source_span: Span = source.into();
        let expected_sub = Some(SpanRef {
            source: &arced_source,
            source_area: 3..6,
        });

        let result = source_span.sub_span(3..6);

        assert_eq!(expected_sub, result);
    }

    #[test]
    fn invalid_subspan_end_outofbounds() {
        let source = Source::new("testing", "abcdefghijklmonpqrstuvw");
        let source_span: Span = source.into();
        let expected_sub = None;

        let result = source_span.sub_span(0..source_span.content().len() + 4);

        assert_eq!(expected_sub, result);
    }
    #[test]
    fn invalid_subspan_start_outofbounds() {
        let source = Source::new("testing", "abcdefghijklmonpqrstuvw");
        let source_span: Span = source.into();
        let expected_sub = None;

        let result =
            source_span.sub_span(source_span.content().len() + 2..source_span.content().len() + 4);

        assert_eq!(expected_sub, result);
    }
}
