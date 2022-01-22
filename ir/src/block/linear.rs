use std::sync::Arc;

use crate::{BasicBlock, InnerBlock};

/// A Linear Iterator that will only return Blocks that form a single Control-Flow, meannig that
/// Headers of Loops or Parts of an If-Statement will not be returned.
/// Once the Control-Flow splits or a Block, that can be reached from multiple locations, is
/// reached, this Iterator will stop and no longer return any new Blocks
pub struct LinearIter {
    current: Option<Arc<InnerBlock>>,
}

impl LinearIter {
    pub(crate) fn new(start: Arc<InnerBlock>) -> Self {
        Self {
            current: Some(start),
        }
    }
}

impl Iterator for LinearIter {
    type Item = BasicBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.current.take()?;
        let result: BasicBlock = raw.into();

        if result.get_predecessors().len() != 1 {
            return None;
        }

        let succs = result.successors();
        if succs.len() != 1 {
            return Some(result);
        }

        let (_, succ) = succs.into_iter().next().unwrap();
        self.current = Some(succ.0);

        Some(result)
    }
}
