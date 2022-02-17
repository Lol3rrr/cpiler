use std::collections::HashSet;

use crate::context::SpillContext;
use crate::spill::replace;

fn linear_iter_inclusive<'v, 'tv, 'r>(
    start: ir::BasicBlock,
    end: ir::BasicBlock,
    visited: &'v mut HashSet<*const ir::InnerBlock>,
    to_visit: &'tv mut Vec<ir::BasicBlock>,
) -> impl Iterator<Item = ir::BasicBlock> + 'r
where
    'v: 'r,
    'tv: 'r,
{
    visited.insert(start.as_ptr());
    to_visit.extend(start.successors().into_iter().map(|(_, b)| b));

    std::iter::from_fn(move || {
        let mut block = to_visit.pop()?;
        while visited.contains(&block.as_ptr()) {
            block = to_visit.pop()?;
        }
        visited.insert(block.as_ptr());

        if block.as_ptr() != end.as_ptr() {
            to_visit.extend(block.successors().into_iter().map(|(_, b)| b));
        }

        Some(block)
    })
}

/// This function simply replaces all the occurances of the previous Variable with new n_var
/// Variable, but does not insert the initial new Load for the new Variable.
///
/// However this may introduce more new Variables for Phi-Nodes, like after certain Conditionals
/// or Loops
pub fn replace(
    start_block: ir::BasicBlock,
    start_index: usize,
    previous: ir::Variable,
    n_var: ir::Variable,
    n_var_block: ir::BasicBlock,
) {
    let block_ctx = SpillContext::determine(start_block.clone());

    let mut starting_stmnts = start_block.get_statements().into_iter();
    let mut result = Vec::new();
    result.extend(starting_stmnts.by_ref().take(start_index));
    result.extend(starting_stmnts.map(|s| replace::statement(s, &previous, &n_var, &n_var_block)));
    start_block.set_statements(result);

    match block_ctx {
        // We will simply go through all the Blocks in the Linear Section and replace all the uses
        // of the previous Variable with the new Variable
        SpillContext::Linear { start, end } => {
            let mut visited: HashSet<_> = vec![start_block.as_ptr()].into_iter().collect();
            let mut to_visit: Vec<_> = start_block
                .successors()
                .into_iter()
                .map(|(_, b)| b)
                .collect();

            while let Some(block) = to_visit.pop() {
                if visited.contains(&block.as_ptr()) {
                    continue;
                }
                visited.insert(block.as_ptr());

                let statements = block.get_statements();
                block.set_statements(
                    statements
                        .into_iter()
                        .map(|s| replace::statement(s, &previous, &n_var, &n_var_block))
                        .collect(),
                );

                if block.as_ptr() == end.as_ptr() {
                    continue;
                }
                to_visit.extend(block.successors().into_iter().map(|(_, b)| b));
            }
        }
        // There are multiple Parts to this
        // 1.
        // Replace the Variable in our current "Part" of the Conditional, so until we reach the
        // common end block, we can just handle it the same way we did in the Linear Context
        //
        // 2.
        // Once we reached the Common Block after the Conditional, we need to determine if the
        // replaced Variable is used after the Conditional itself, or if there already exists a
        // newer Version of it or the Variable is just never accessed again.
        // If the Variable is never used again or overwritten by a newer definition, we can just
        // stop right here and we are done.
        // Otherwise we need to check if there exists a Phi-Node for this Variable already, if
        // there is go to 2a. otherwise go to 2b.
        //
        // 2a.
        // Because there already exists a Phi-Node for the Variable we want to replace, we can
        // simply update the Phi-Node to now use the new Variable and everything works as expected
        //
        // 2b.
        // We then need to insert a new Phi-Node at the beginning of the Common-Block and then
        // start the replace Process again starting at the common Block, in which we now replace
        // the Old-Variable with the target of the newly inserted Phi-Node
        SpillContext::Conditional {
            header,
            end,
            current_start,
            current_end,
            other_start,
        } => {
            // 1.
            let mut visited = HashSet::new();
            let mut to_visit = Vec::new();
            let linear_inner_iter =
                linear_iter_inclusive(start_block, current_end, &mut visited, &mut to_visit)
                    .skip(1);
            for block in linear_inner_iter {
                let statements = block.get_statements();
                block.set_statements(
                    statements
                        .into_iter()
                        .map(|s| replace::statement(s, &previous, &n_var, &n_var_block))
                        .collect(),
                );
            }

            // 2.
            let common_statements = end.get_statements();
            let phi_statement = common_statements.iter().enumerate().find(|(_, s)| match s {
                ir::Statement::Assignment {
                    target,
                    value: ir::Value::Phi { .. },
                } => target.name == n_var.name,
                _ => false,
            });

            match phi_statement {
                // 2a
                Some((index, stmnt)) => {
                    todo!("Update Phi Statement")
                }
                // 2b
                None => {
                    let used_after = end
                        .block_iter()
                        .flat_map(|b| b.block_used_vars())
                        .any(|(var, _)| var.name == previous.name);
                    if !used_after {
                        return;
                    }

                    todo!("Insert new Phi Statement")
                }
            };
        }
        // 1.
        // Replace the Variable inside the Loop just as you would with in the Linear Section until
        // we have reached the Header again, which we need to treat differently
        //
        // 2.
        // We now need to check if there already exists a Phi-Node for the Variable, if that is the
        // Case go to 2a otherwise if there is no Phi-Node go to 2b.
        //
        // 2a.
        // The Phi-Node already exists so now we only need to update the Sources for the Phi-Node
        // if they have changed. After this we are already done.
        //
        // 2b.
        // We now have to insert a new Phi-Node in the Head-Block. However we then also need to
        // replace Variable that came from before the Loop with the new Phi-Variable in the loop
        // itself, which can just be done like Step 1.
        // After that we also need to update all the uses after the Loop so we need to determine
        // the Context of the Block after the Loop and then start the replacement for that context
        // where we replace the Variable from before the Loop with the new Phi-Variable
        SpillContext::Loop {
            header,
            first_inner,
            first_after,
        } => {
            todo!("Replace Loop")
        }
    };
}
