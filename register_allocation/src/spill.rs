use std::collections::HashSet;

use crate::context::{conditional_spill, SpillContext, SpillResult};

mod conditional;
mod linear;
mod replace;
mod replacement;

/// To spill a Variable we will in Principle create a new Generation of the Variable and
/// only assign it just before the spilled Variable would be first used after the Spill.
/// From that Point on we then need to replace all the uses of the Spilled Variable with the new
/// Generation
///
/// We need to consider three major situations:
/// * There is no Control-Flow in the Program
/// * There are branches in the Program but no loops
/// * There are loops in the Program
pub fn spill_variable(
    largest_vars: HashSet<ir::Variable>,
    start_block: ir::BasicBlock,
    start_index: usize,
    spill_ctx: SpillContext,
) {
    let spill_var = spill_ctx.determine_spill_var(largest_vars, start_block.clone(), start_index);
    let replacement = spill_var.replacement();

    match spill_ctx {
        SpillContext::Linear { start, end } => {
            let spill_res = match spill_var {
                SpillResult::Linear(res) => res,
                other => {
                    unreachable!("This should never happen")
                }
            };

            linear::spill(start.clone(), spill_res, replacement);
        }
        SpillContext::Conditional {
            header,
            end,
            current_start,
            current_end,
            other_start,
        } => {
            let cond_spill_res = match spill_var {
                SpillResult::Conditional(cond) => cond,
                _ => {
                    todo!()
                }
            };

            let (start_block, start_index, to_replace, n_var, n_var_block) = match cond_spill_res {
                conditional_spill::SpillResult::OuterVariable { var } => {
                    conditional::spill_outer(
                        header.clone(),
                        end.clone(),
                        var.clone(),
                        replacement.clone(),
                    );
                    (end.clone(), 1, var, replacement, end)
                }
                conditional_spill::SpillResult::InnerVariable {
                    var,
                    load_block,
                    load_index,
                } => {
                    dbg!(
                        start_block.as_ptr(),
                        load_block.as_ptr(),
                        start_index,
                        load_index
                    );
                    conditional::spill_inner(
                        start_block.clone(),
                        start_index,
                        load_block.clone(),
                        load_index,
                        var.clone(),
                        replacement.clone(),
                    );

                    (start_block, start_index, var, replacement, load_block)
                }
            };

            replacement::replace(start_block, start_index, to_replace, n_var, n_var_block);
        }
        SpillContext::Loop {
            header,
            first_inner,
            first_after,
        } => {
            todo!("Spill in Loop")
        }
    };
}

pub fn save_statement(var: ir::Variable) -> ir::Statement {
    if var.global() {
        todo!()
    } else {
        ir::Statement::SaveVariable { var }
    }
}

pub fn load_statement(var: ir::Variable) -> ir::Statement {
    if var.global() {
        todo!()
    } else {
        ir::Statement::Assignment {
            target: var,
            value: ir::Value::Unknown,
        }
    }
}
