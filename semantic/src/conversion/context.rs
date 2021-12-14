use ir::BasicBlock;

#[derive(Debug)]
pub struct ConvertContext {
    loop_ctx: Option<(BasicBlock, BasicBlock)>,
}

impl ConvertContext {
    pub fn new() -> Self {
        Self { loop_ctx: None }
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
        }
    }
}
