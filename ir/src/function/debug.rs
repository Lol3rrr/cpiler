use std::{collections::HashSet, fmt::Debug};

use crate::BasicBlock;

pub struct DebugBlocks {
    pub start: BasicBlock,
}

impl Debug for DebugBlocks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f_blocks = f.debug_struct("Blocks");

        let mut blocks_printed = HashSet::new();
        let mut blocks_left = vec![self.start.clone()];
        while let Some(block) = blocks_left.pop() {
            let name = format!("0x{:x}", block.as_ptr() as usize);

            f_blocks.field(&name, &block);

            for (ptr, block) in block.successors() {
                if blocks_printed.contains(&ptr) {
                    continue;
                }

                blocks_printed.insert(ptr);
                blocks_left.push(block);
            }
        }
        f_blocks.finish()?;

        Ok(())
    }
}
