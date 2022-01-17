use std::{fmt::Debug, sync::Weak};

use crate::{BasicBlock, InnerBlock};

/// A WeakPtr to an InnerBlock/BasicBlock, this is needed to break the Ownership Problem of Graphs
/// that may contain cycles
#[derive(Clone)]
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

impl PartialEq for WeakBlockPtr {
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl Debug for WeakBlockPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}", self.as_ptr() as usize)
    }
}
