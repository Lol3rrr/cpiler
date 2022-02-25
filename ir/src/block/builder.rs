use std::sync::{Arc, RwLock};

use crate::{BasicBlock, InnerBlock, Statement, WeakBlockPtr};

/// This is a more convient way to create Blocks
pub struct BlockBuilder {
    predecessors: Vec<WeakBlockPtr>,
    parts: Vec<Statement>,
    description: Option<String>,
}

impl BlockBuilder {
    /// Creates a new Builder with these most essential Parts that are always needed
    pub fn new(predecessors: Vec<WeakBlockPtr>, parts: Vec<Statement>) -> Self {
        Self {
            predecessors,
            parts,
            description: None,
        }
    }

    /// Sets the Description of the Block
    pub fn description<S>(mut self, description: S) -> Self
    where
        S: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    /// Actually builds the Block
    pub fn build(self) -> BasicBlock {
        Arc::new(InnerBlock {
            predecessor: RwLock::new(self.predecessors),
            parts: RwLock::new(self.parts),
            description: self.description,
        })
        .into()
    }
}
