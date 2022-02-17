use std::sync::{Arc, RwLock};

use crate::{BasicBlock, InnerBlock, Statement, WeakBlockPtr};

pub struct BlockBuilder {
    predecessors: Vec<WeakBlockPtr>,
    parts: Vec<Statement>,
    description: Option<String>,
}

impl BlockBuilder {
    pub fn new(predecessors: Vec<WeakBlockPtr>, parts: Vec<Statement>) -> Self {
        Self {
            predecessors,
            parts,
            description: None,
        }
    }

    pub fn description<S>(mut self, description: S) -> Self
    where
        S: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    pub fn build(self) -> BasicBlock {
        Arc::new(InnerBlock {
            predecessor: RwLock::new(self.predecessors),
            parts: RwLock::new(self.parts),
            description: self.description,
        })
        .into()
    }
}
