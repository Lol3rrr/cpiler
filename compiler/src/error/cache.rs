use std::collections::HashMap;

use ariadne::{Cache, Source};

pub struct SourceCache {
    sources: HashMap<String, Source>,
}

impl SourceCache {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    fn contains_source(&self, source: &general::Source) -> bool {
        self.sources.contains_key(source.name())
    }

    pub fn add_source(&mut self, span: &general::Span) {
        let tmp_source = span.source();
        let name = tmp_source.name().to_string();
        let content = tmp_source.content().to_string();
        let source = Source::from(content);

        self.sources.insert(name, source);
    }
}

impl<const N: usize> From<[&general::Span; N]> for SourceCache {
    fn from(sources: [&general::Span; N]) -> Self {
        let mut tmp = Self::new();

        for tmp_s in sources {
            if tmp.contains_source(tmp_s.source()) {
                continue;
            }

            tmp.add_source(tmp_s);
        }

        tmp
    }
}

impl Cache<str> for SourceCache {
    fn fetch(&mut self, id: &str) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        match self.sources.get(id) {
            Some(raw_source) => Ok(raw_source),
            None => Err(Box::new(format!("Unknown Source with ID: {:?}", id))),
        }
    }

    fn display<'a>(&self, id: &'a str) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(id.to_string()))
    }
}

impl Cache<&str> for SourceCache {
    fn fetch(&mut self, id: &&str) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        Cache::<str>::fetch(self, *id)
    }

    fn display<'a>(&self, id: &'a &str) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Cache::<str>::display(self, *id)
    }
}

impl Cache<&general::Span> for SourceCache {
    fn fetch(&mut self, id: &&general::Span) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        let source = id.source();
        Cache::<str>::fetch(self, source.name())
    }

    fn display<'a>(&self, id: &'a &general::Span) -> Option<Box<dyn std::fmt::Display + 'a>> {
        let source = id.source();
        Cache::<str>::display(self, source.name())
    }
}
