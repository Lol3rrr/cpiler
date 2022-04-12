use std::sync::RwLock;

use crate::{Statement, WeakBlockPtr};

/// This actually contains the Data for a Block, but is never directly used and cant be constructed
/// or really obtained, because it is always behind some sort of wrapper type
#[derive(Debug)]
// We allow dead code here because of the description field which is useful for debugging
#[allow(dead_code)]
pub struct InnerBlock {
    /// The List of Predecessors from which you can jump to this Block
    pub(crate) predecessor: RwLock<Vec<WeakBlockPtr>>,
    /// The actual Statements in this Block
    pub(crate) parts: RwLock<Vec<Statement>>,

    /// An optional Description of the Block or its purpose to help in debugging and other
    /// tasks
    pub(crate) description: Option<String>,
}
