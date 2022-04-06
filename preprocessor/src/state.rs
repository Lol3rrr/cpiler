use std::collections::HashSet;

use crate::resolver::DefineManager;

#[derive(Debug)]
pub struct State {
    pub defines: DefineManager,
    included: HashSet<String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            defines: DefineManager::new(),
            included: HashSet::new(),
        }
    }

    pub fn is_file_included(&self, path: &str) -> bool {
        self.included.contains(path)
    }
    pub fn add_included_file(&mut self, path: String) {
        self.included.insert(path);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
