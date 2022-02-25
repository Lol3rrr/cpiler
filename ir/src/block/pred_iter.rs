use std::collections::HashSet;

use crate::{BasicBlock, InnerBlock};

/// An iterator over the Predecessors of a Block
pub struct PredecessorIterator {
    visited: HashSet<*const InnerBlock>,
    to_visit: Vec<BasicBlock>,
}

impl PredecessorIterator {
    /// Creates a new Iterator with the given Block as the starting Point
    pub fn new(start: BasicBlock) -> Self {
        Self {
            visited: HashSet::new(),
            to_visit: vec![start],
        }
    }
}

impl Iterator for PredecessorIterator {
    type Item = BasicBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tmp = self.to_visit.pop()?;
        while self.visited.contains(&tmp.as_ptr()) {
            tmp = self.to_visit.pop()?;
        }

        let block = tmp;
        self.visited.insert(block.as_ptr());

        self.to_visit.extend(
            block
                .get_predecessors()
                .into_iter()
                .filter_map(|p| p.upgrade()),
        );

        Some(block)
    }
}
