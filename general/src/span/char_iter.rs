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
        let elem = content.chars().nth(self.current)?;

        let result = (self.current, elem);

        self.current += 1;

        Some(result)
    }
}
