use std::ops::Deref;

use crate::Span;

pub struct CharIndexIter<S> {
    span: S,
    current: usize,
}

impl<S> CharIndexIter<S> {
    pub fn new(span: S) -> Self {
        Self { span, current: 0 }
    }
}

impl<S> Iterator for CharIndexIter<S>
where
    S: Deref<Target = Span>,
{
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let content = self.span.deref().content();
        let elem = content.char_indices().nth(self.current)?;

        self.current += 1;

        Some(elem)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::Source;

    use super::*;

    #[test]
    fn ascii() {
        let content = "test";
        let source = Source::new("input", content);
        let span: Arc<Span> = Arc::new(source.into());
        let mut iter = CharIndexIter::new(span.clone());

        assert_eq!(Some((0, 't')), iter.next());
    }

    #[test]
    fn unicode() {
        let content = "y̆a";
        let source = Source::new("input", content);
        let span: Arc<Span> = Arc::new(source.into());
        let mut iter = CharIndexIter::new(span.clone());

        assert_eq!(Some((0, 'y')), iter.next());
        assert_eq!(Some((1, '\u{0306}')), iter.next());
        assert_eq!(Some((3, 'a')), iter.next());
    }
}
