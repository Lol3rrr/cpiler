use std::{fmt::Debug, ops::Range, sync::Arc};

#[derive(PartialEq, Clone)]
pub struct Source {
    name: String,
    content: String,
}

impl Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source {{ name = {} }}", self.name)
    }
}

impl Source {
    pub fn new<N, C>(name: N, content: C) -> Self
    where
        N: Into<String>,
        C: Into<String>,
    {
        Self {
            name: name.into(),
            content: content.into(),
        }
    }

    pub fn get(&self, range: Range<usize>) -> Option<&str> {
        self.content.get(range)
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn sub_content(&self, range: Range<usize>) -> Option<&str> {
        self.content.get(range)
    }
}
