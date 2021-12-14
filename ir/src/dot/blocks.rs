use std::collections::HashSet;

pub struct DrawnBlocks {
    inner: HashSet<String>,
}

impl DrawnBlocks {
    pub fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    pub fn add_block<T>(&mut self, name: T)
    where
        T: Into<String>,
    {
        self.inner.insert(name.into());
    }

    pub fn contains<T>(&self, name: T) -> bool
    where
        T: AsRef<str>,
    {
        self.inner.contains(name.as_ref())
    }
}

impl Default for DrawnBlocks {
    fn default() -> Self {
        Self::new()
    }
}
