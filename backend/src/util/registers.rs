//! Handles all the Register allocation and the like
// This is build around this Paper
// https://link.springer.com/content/pdf/10.1007%2F11688839_20.pdf

use std::{collections::HashMap, fmt::Debug, hash::Hash, path::PathBuf};

use ir::Variable;

/// This will perform the Register Allocation and spilling
pub fn allocate_registers<R>(
    func: &ir::FunctionDefinition,
    registers: &[R],
    build_path: Option<PathBuf>,
) -> HashMap<Variable, R>
where
    R: Clone + Debug + Hash + PartialEq + Eq + register_allocation::Register,
{
    register_allocation::RegisterMapping::allocate(
        func,
        registers,
        register_allocation::AllocationCtx { build_path },
    )
    .into()
}
