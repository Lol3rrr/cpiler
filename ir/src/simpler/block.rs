use std::collections::HashMap;

use super::{BlockIndex, Statement};

/// A single Basic Block in the simpler IR
#[derive(Debug, Clone)]
pub struct Block {
    /// The Predecessors of this Block
    pub predecessors: Vec<BlockIndex>,
    /// The actual Statements in this Block
    pub statments: Vec<Statement>,
}

impl Block {
    pub(crate) fn convert_from_complex(
        &mut self,
        src: crate::BasicBlock,
        block_map: &HashMap<*const crate::InnerBlock, BlockIndex>,
    ) {
        let predecessors = src.get_predecessors().into_iter().filter_map(|pred| {
            let index = block_map.get(&pred.as_ptr())?;
            Some(index.clone())
        });

        let statements = src
            .get_statements()
            .into_iter()
            .map(|stmnt| Statement::from_complex(stmnt, block_map));

        self.predecessors = predecessors.collect();
        self.statments = statements.collect();
    }
}
