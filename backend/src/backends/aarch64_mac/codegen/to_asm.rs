use ir::BasicBlock;

use crate::backends::aarch64_mac::asm;

use super::{statement, Context};

pub mod binaryop;
pub mod function_call;
pub mod load;
pub mod unaryop;
pub mod write;

pub fn block_name(block: &BasicBlock) -> String {
    format!("block_{:x}", block.as_ptr() as usize)
}

pub fn block_to_asm(block: BasicBlock, ctx: &Context) -> asm::Block {
    let statements = block.get_statements();

    let name = block_name(&block);
    let mut instructions = Vec::new();

    for stmnt in statements {
        let stmnt_instr = statement::to_asm(stmnt, ctx);
        instructions.extend(stmnt_instr);
    }

    asm::Block { name, instructions }
}
