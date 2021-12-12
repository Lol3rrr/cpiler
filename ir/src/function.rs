use std::{collections::HashSet, fmt::Debug, sync::Arc};

use crate::{BasicBlock, Type};

/// A definition of a Function
#[derive(Clone, PartialEq)]
pub struct FunctionDefinition {
    /// The Arguments of the Function in the Order they will be received in
    pub arguments: Vec<(String, Type)>,
    /// The initial BasicBlock of the Function
    pub block: Arc<BasicBlock>,
    /// The Return Type of the Function
    pub return_ty: Type,
}

impl FunctionDefinition {
    /// Creates the corresponding Dot Graphviz String for this
    pub fn to_dot(
        &self,
        name: &str,
        lines: &mut Vec<String>,
        drawn: &mut HashSet<*const BasicBlock>,
    ) {
        let block_name = self.block.to_dot(lines, drawn);

        lines.push(format!("func_{} -> {}", name, block_name));
    }
}

impl Debug for FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Arguments: {:?}", self.arguments)?;
        writeln!(f, "Return-Type: {:?}", self.return_ty)?;

        let mut f_blocks = f.debug_struct("Blocks");

        let mut blocks_printed = HashSet::new();
        let mut blocks_left = vec![self.block.clone()];
        while let Some(block) = blocks_left.pop() {
            let name = format!("0x{:x}", Arc::as_ptr(&block) as usize);

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
