use std::sync::RwLock;

use crate::{Statement, WeakBlockPtr};

#[derive(Debug)]
pub struct InnerBlock {
    /// The List of Predecessors from which you can jump to this Block
    pub(crate) predecessor: RwLock<Vec<WeakBlockPtr>>,
    /// The actual Statements in this Block
    pub(crate) parts: RwLock<Vec<Statement>>,
}
