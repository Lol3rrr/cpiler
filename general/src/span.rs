use std::ops::Range;

// TODO
// Potentially switch the source String to an Arc<String> and then whenever
// you construct a subspan you dont clone the String itself but rather only the
// Arc which should be faster and more efficient

mod char_iter;
pub use char_iter::CharIndexIter;

/// A Span describes a Part of some overall String, most likely source Code
#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    source: String,
    source_area: Range<usize>,
    content: String,
}

impl Span {
    pub fn new_source<S, C>(source: S, content: C) -> Self
    where
        S: Into<String>,
        C: Into<String>,
    {
        let source = source.into();
        let content = content.into();

        Self {
            source,
            source_area: 0..content.len(),
            content,
        }
    }

    pub fn from_parts<S, C>(source: S, content: C, range: Range<usize>) -> Self
    where
        S: Into<String>,
        C: Into<String>,
    {
        Self {
            source: source.into(),
            source_area: range,
            content: content.into(),
        }
    }

    pub fn sub_span<'out>(&'out self, range: Range<usize>) -> Option<SpanRef<'out>> {
        if range.start > self.content.len() || range.end > self.content.len() {
            return None;
        }

        let sub_content = &self.content[range.clone()];

        let source_start = self.source_area.start;
        let sub_area = source_start + range.start..source_start + range.end;

        Some(SpanRef {
            source: &self.source,
            source_area: sub_area,
            content: sub_content,
        })
    }

    pub fn content(&'_ self) -> &'_ str {
        &self.content
    }
    pub fn source(&self) -> &str {
        &self.source
    }
}

impl AsRef<str> for Span {
    fn as_ref(&self) -> &str {
        self.content()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SpanRef<'a> {
    source: &'a str,
    source_area: Range<usize>,
    content: &'a str,
}

impl<'a> SpanRef<'a> {
    pub fn content(&self) -> &str {
        self.content
    }

    pub fn sub_span<'out>(&'out self, range: Range<usize>) -> Option<SpanRef<'out>> {
        if range.start > self.content.len() || range.end > self.content.len() {
            return None;
        }

        let sub_content = &self.content[range.clone()];

        let source_start = self.source_area.start;
        let sub_area = source_start + range.start..source_start + range.end;

        Some(SpanRef {
            source: &self.source,
            source_area: sub_area,
            content: sub_content,
        })
    }
}

impl<'s> Into<Span> for SpanRef<'s> {
    fn into(self) -> Span {
        Span {
            source: self.source.to_owned(),
            source_area: self.source_area,
            content: self.content.to_owned(),
        }
    }
}
impl<'s> Into<Span> for &SpanRef<'s> {
    fn into(self) -> Span {
        Span {
            source: self.source.to_owned(),
            source_area: self.source_area.clone(),
            content: self.content.to_owned(),
        }
    }
}
impl<'o, 's> From<&'o Span> for SpanRef<'s>
where
    'o: 's,
{
    fn from(source: &'o Span) -> Self {
        Self {
            source: source.source(),
            content: source.content(),
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
        let source_span = Span::new_source("testing", "abcdefghijklmonpqrstuvw");
        let expected_sub = Some(SpanRef {
            source: "testing",
            source_area: 0..6,
            content: "abcdef",
        });

        let result = source_span.sub_span(0..6);

        assert_eq!(expected_sub, result);
    }

    #[test]
    fn valid_subspan_middle() {
        let source_span = Span::new_source("testing", "abcdefghijklmonpqrstuvw");
        let expected_sub = Some(SpanRef {
            source: "testing",
            source_area: 3..6,
            content: "def",
        });

        let result = source_span.sub_span(3..6);

        assert_eq!(expected_sub, result);
    }

    #[test]
    fn invalid_subspan_end_outofbounds() {
        let source_span = Span::new_source("testing", "abcdefghijklmonpqrstuvw");
        let expected_sub = None;

        let result = source_span.sub_span(0..source_span.content.len() + 4);

        assert_eq!(expected_sub, result);
    }
    #[test]
    fn invalid_subspan_start_outofbounds() {
        let source_span = Span::new_source("testing", "abcdefghijklmonpqrstuvw");
        let expected_sub = None;

        let result =
            source_span.sub_span(source_span.content.len() + 2..source_span.content.len() + 4);

        assert_eq!(expected_sub, result);
    }
}
