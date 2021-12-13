use std::collections::HashMap;

use crate::InnerBlock;

pub trait CompareGraph {
    /// blocks: Contains all the already Visited Blocks and at which place in the Graph Traversel
    /// they were found
    /// current_block: The Number of the current Block in the Graph Traversel
    fn compare(
        &self,
        other: &Self,
        blocks: &mut HashMap<*const InnerBlock, usize>,
        current_block: usize,
    ) -> bool;
}
