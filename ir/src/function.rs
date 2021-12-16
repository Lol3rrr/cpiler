use std::{collections::HashSet, fmt::Debug};

use general::dot;

use crate::{BasicBlock, ToDot, Type};

/// A definition of a Function
#[derive(Clone, PartialEq)]
pub struct FunctionDefinition {
    /// The Name of the Function
    pub name: String,
    /// The Arguments of the Function in the Order they will be received in
    pub arguments: Vec<(String, Type)>,
    /// The initial BasicBlock of the Function
    pub block: BasicBlock,
    /// The Return Type of the Function
    pub return_ty: Type,
}

struct DebugBlocks {
    start: BasicBlock,
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

impl Debug for FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f_struct = f.debug_struct("FunctionDefinition");

        let dbg_blocks = DebugBlocks {
            start: self.block.clone(),
        };

        f_struct.field("arguments", &self.arguments);
        f_struct.field("return_ty", &self.return_ty);
        f_struct.field("blocks", &dbg_blocks);

        f_struct.finish()?;

        Ok(())
    }
}

impl ToDot for FunctionDefinition {
    fn to_dot(
        &self,
        lines: &mut dyn dot::Graph,
        drawn: &mut crate::dot::DrawnBlocks,
        ctx: &crate::dot::Context,
    ) -> String {
        let dot_name = format!("func_{}", self.name);
        let mut function_graph = dot::SubGraph::new(&dot_name)
            .cluster()
            .arg("label", format!("Function-{}", self.name));

        let block_name = self.block.to_dot(&mut function_graph, drawn, ctx);
        lines.add_subgraph(function_graph);

        lines.add_edge(dot::Edge::new(&dot_name, block_name));

        dot_name
    }

    fn name(&self, _: &crate::dot::Context) -> String {
        format!("func_{}", self.name)
    }
}
