use std::sync::{atomic, Arc};

use ir::BasicBlock;

#[derive(Debug)]
pub struct ConvertContext {
    loop_ctx: Option<(BasicBlock, BasicBlock)>,
    current_tmp: Arc<atomic::AtomicUsize>,
}

impl ConvertContext {
    pub fn base() -> Self {
        Self {
            loop_ctx: None,
            current_tmp: Arc::new(atomic::AtomicUsize::new(0)),
        }
    }

    pub fn next_tmp(&self) -> usize {
        self.current_tmp.fetch_add(1, atomic::Ordering::SeqCst)
    }

    pub fn get_loop_start(&self) -> Option<&BasicBlock> {
        let (start, _) = self.loop_ctx.as_ref()?;
        Some(start)
    }
    pub fn get_loop_end(&self) -> Option<&BasicBlock> {
        let (_, end) = self.loop_ctx.as_ref()?;
        Some(end)
    }

    pub fn with_loop(&self, start: BasicBlock, end: BasicBlock) -> Self {
        Self {
            loop_ctx: Some((start, end)),
            current_tmp: self.current_tmp.clone(),
        }
    }
}
