use std::sync::Weak;

use crate::{BasicBlock, InnerBlock};

/// A WeakPtr to an InnerBlock/BasicBlock, this is needed to break the Ownership Problem of Graphs
/// that may contain cycles
#[derive(Debug, Clone)]
pub struct WeakBlockPtr(Weak<InnerBlock>);

impl WeakBlockPtr {
    /// Attempts to upgrade this WeakPointer into an actual BasicBlock Instance
    pub fn upgrade(&self) -> Option<BasicBlock> {
        let inner = self.0.upgrade()?;
        Some(inner.into())
    }

    /// Gets the Pointer to the underlying Data
    pub fn as_ptr(&self) -> *const InnerBlock {
        self.0.as_ptr()
    }
}

impl From<Weak<InnerBlock>> for WeakBlockPtr {
    fn from(inner: Weak<InnerBlock>) -> Self {
        Self(inner)
    }
}
