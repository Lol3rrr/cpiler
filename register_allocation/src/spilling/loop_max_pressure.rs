use std::collections::HashSet;

use ir::InnerBlock;

use super::RegisterConfig;

// TODO
// This definetly needs more work on it
pub fn max_pressure(func: &ir::FunctionDefinition, head: *const InnerBlock) -> RegisterConfig {
    let graph = func.to_directed_graph();

    let mut root_chain = graph.chain_from(head);

    let _ = root_chain.next_entry();

    let inner = match root_chain.next_entry() {
        Some(graphs::directed::ChainEntry::Cycle { inner, .. }) => inner,
        _ => {
            todo!("Unexpected ChainEntry")
        }
    }
    .flatten();

    let used_vars: HashSet<_> = inner
        .flat_map(|b| b.get_statements())
        .flat_map(|s| s.used_vars())
        .collect();

    RegisterConfig {
        general_purpose_count: used_vars.iter().filter(|v| !v.ty.is_float()).count(),
        floating_point_count: used_vars.iter().filter(|v| v.ty.is_float()).count(),
    }
}
