use std::{collections::HashSet, sync::Arc};

use crate::{BasicBlock, Type};

/// A definition of a Function
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    /// The Arguments of the Function in the Order they will be received in
    pub arguments: Vec<(String, Type)>,
    /// The initial BasicBlock of the Function
    pub block: Arc<BasicBlock>,
    /// The Return Type of the Function
    pub return_ty: Type,
}

impl FunctionDefinition {
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
