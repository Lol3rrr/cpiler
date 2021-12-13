use std::collections::HashSet;

pub struct DrawnBlocks {
    inner: HashSet<*const ()>,
}

impl DrawnBlocks {
    pub fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    pub fn add_block<T>(&mut self, ptr: *const T) {
        let ptr = ptr as *const ();
        self.inner.insert(ptr);
    }

    pub fn contains<T>(&self, ptr: *const T) -> bool {
        let ptr = ptr as *const ();
        self.inner.contains(&ptr)
    }
}

impl Default for DrawnBlocks {
    fn default() -> Self {
        Self::new()
    }
}
