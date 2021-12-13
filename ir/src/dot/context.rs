use std::collections::HashMap;

pub struct Context {
    inner: HashMap<&'static str, Box<dyn std::any::Any>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn set<V>(&mut self, name: &'static str, value: V)
    where
        V: 'static,
    {
        self.inner.insert(name, Box::new(value));
    }

    pub fn get(&self, name: &'static str) -> Option<&Box<dyn std::any::Any>> {
        self.inner.get(name)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
