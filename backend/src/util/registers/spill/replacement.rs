use std::collections::HashSet;

use crate::util::registers::{context::SpillContext, spill::replace};

pub fn replace(
    start_block: ir::BasicBlock,
    start_index: usize,
    previous: ir::Variable,
    n_var: ir::Variable,
) {
    let block_ctx = SpillContext::determine(start_block.clone());

    match block_ctx {
        SpillContext::Linear { start, end } => {
            let mut starting_stmnts = start_block.get_statements().into_iter();
            let mut result = Vec::new();
            result.extend(starting_stmnts.by_ref().take(start_index));
            result.extend(starting_stmnts.map(|s| replace::statement(s, &previous, &n_var)));
            start_block.set_statements(result);

            let mut visited: HashSet<_> = vec![start_block.as_ptr()].into_iter().collect();
            let mut to_visit: Vec<_> = start_block
                .successors()
                .into_iter()
                .map(|(_, b)| b)
                .collect();

            while let Some(block) = to_visit.pop() {
                if block.as_ptr() == end.as_ptr() {
                    continue;
                }
                if visited.contains(&block.as_ptr()) {
                    continue;
                }
                visited.insert(block.as_ptr());

                let statements = block.get_statements();
                block.set_statements(
                    statements
                        .into_iter()
                        .map(|s| replace::statement(s, &previous, &n_var))
                        .collect(),
                );

                to_visit.extend(block.successors().into_iter().map(|(_, b)| b));
            }
        }
        SpillContext::Conditional {
            header,
            end,
            current_start,
            other_start,
        } => {
            todo!("Replace Conditional")
        }
        SpillContext::Loop {
            header,
            first_inner,
            first_after,
        } => {
            todo!("Replace Loop")
        }
    };
}
