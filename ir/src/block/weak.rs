use std::sync::Weak;

use crate::{BasicBlock, InnerBlock};

#[derive(Debug, Clone)]
pub struct WeakBlockPtr(Weak<InnerBlock>);

impl WeakBlockPtr {
    pub fn upgrade(&self) -> Option<BasicBlock> {
        let inner = self.0.upgrade()?;
        Some(inner.into())
    }

    pub fn as_ptr(&self) -> *const InnerBlock {
        self.0.as_ptr()
    }
}

impl From<Weak<InnerBlock>> for WeakBlockPtr {
    fn from(inner: Weak<InnerBlock>) -> Self {
        Self(inner)
    }
}
