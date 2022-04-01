use std::collections::BTreeMap;

use ir::{BasicBlock, InnerBlock};

use super::Reload;

#[derive(Debug)]
pub struct ReloadList {
    data: BTreeMap<*const InnerBlock, (BasicBlock, Vec<Reload>)>,
}

impl ReloadList {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, block: BasicBlock, data: Vec<Reload>) {
        match self.data.get_mut(&block.as_ptr()) {
            Some((_, prev)) => {
                prev.extend(data);
            }
            None => {
                self.data.insert(block.as_ptr(), (block, data));
            }
        };
    }

    pub fn into_iter(self) -> impl Iterator<Item = (BasicBlock, Vec<Reload>)> {
        self.data.into_values()
    }
}
