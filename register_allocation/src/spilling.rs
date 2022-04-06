//! This is based on this Paper https://link.springer.com/content/pdf/10.1007%252F978-3-642-00722-4_13.pdf

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    vec,
};

use crate::debug_ctx::DebugContext;

mod min;
use ir::{InnerBlock, WeakBlockPtr};
use min::min_algorithm;

mod reload_list;
use self::reload_list::ReloadList;

fn inblock_distance(block: &ir::BasicBlock) -> (HashMap<ir::Variable, usize>, usize) {
    let statements = block.get_statements();
    (
        statements
            .iter()
            .enumerate()
            .rev()
            .flat_map(|(i, s)| s.used_vars().into_iter().map(move |v| (v, i)))
            .collect(),
        statements.len(),
    )
}

fn merge_distances<I>(target: &mut HashMap<ir::Variable, usize>, other: I)
where
    I: Iterator<Item = (ir::Variable, usize)>,
{
    for (var, n_distance) in other {
        match target.get_mut(&var) {
            Some(p_distance) => {
                if *p_distance >= n_distance {
                    *p_distance = n_distance;
                }
            }
            None => {
                target.insert(var, n_distance);
            }
        };
    }
}

fn next_use_distances(
    root: &ir::BasicBlock,
    visited: &mut HashSet<*const ir::InnerBlock>,
) -> HashMap<ir::Variable, usize> {
    if visited.contains(&root.as_ptr()) {
        return HashMap::new();
    }

    let (root_distances, max_root) = inblock_distance(root);
    visited.insert(root.as_ptr());

    let mut distances = root_distances;

    let current = root.clone();
    let succs = current.successors();
    if succs.is_empty() {
    } else if succs.len() == 1 {
        let succ = succs.into_values().next().unwrap();

        let succ_distances = next_use_distances(&succ, visited);

        merge_distances(
            &mut distances,
            succ_distances
                .into_iter()
                .map(|(k, v)| (k, v + max_root + 1)),
        );
    } else if succs.len() == 2 {
        //let preds = current.get_predecessors();

        let succ_distances = succs.into_values().map(|b| next_use_distances(&b, visited));
        for tmp_dist in succ_distances {
            merge_distances(
                &mut distances,
                tmp_dist.into_iter().map(|(k, v)| (k, v + max_root + 1)),
            );
        }
        /*
        if preds.len() == 1 {

        } else {

            todo!("Loop")
        }
        */
    } else {
        todo!("More than 2 Successors")
    }

    distances
}

#[derive(Debug, Clone)]
pub struct Reload {
    previous: ir::Variable,
    var: ir::Variable,
    position: usize,
}

fn replace_operand(oper: &mut ir::Operand, previous: &ir::Variable, n_var: &ir::Variable) {
    match oper {
        ir::Operand::Variable(var) if var == previous => {
            *var = n_var.clone();
        }
        _ => {}
    };
}

#[derive(Debug)]
enum PrevDefinition {
    Single(ir::Variable),
    Mutliple(Vec<(ir::Variable, ir::BasicBlock)>),
}
fn find_previous_definition<SI>(
    preds: &[ir::WeakBlockPtr],
    statements: SI,
    var_name: &str,
) -> PrevDefinition
where
    SI: Iterator<Item = ir::Statement>,
{
    let mut last_def = None;
    for stmnt in statements {
        if let ir::Statement::Assignment { target, .. } = stmnt {
            if target.name == var_name {
                last_def = Some(target.clone());
            }
        }
    }
    if let Some(var) = last_def {
        return PrevDefinition::Single(var);
    }

    match core::cmp::Ord::cmp(&preds.len(), &1) {
        core::cmp::Ordering::Equal => {
            let weak_pred = preds.iter().next().unwrap();
            let pred = weak_pred.upgrade().unwrap();

            let n_preds = pred.get_predecessors();
            let statements = pred.get_statements();

            let pred_def =
                find_previous_definition(&n_preds, statements.into_iter().rev(), var_name);

            if let PrevDefinition::Single(_) = &pred_def {
                return pred_def;
            }

            dbg!(&pred_def);

            todo!("Find Variable in Single Predecessor");
        }
        core::cmp::Ordering::Greater => {
            let found_vars = preds
                .iter()
                .map(|raw_pred| {
                    let pred = raw_pred.upgrade().unwrap();

                    let pred_preds = pred.get_predecessors();

                    let pred_var = find_previous_definition(
                        &pred_preds,
                        pred.get_statements().into_iter(),
                        var_name,
                    );

                    match pred_var {
                        PrevDefinition::Single(var) => (var, pred),
                        PrevDefinition::Mutliple(_) => todo!(""),
                    }
                })
                .collect();

            PrevDefinition::Mutliple(found_vars)
        }
        core::cmp::Ordering::Less => {
            todo!()
        }
    }
}

fn replace_used_variables(
    stmnt: &mut ir::Statement,
    previous: &ir::Variable,
    n_var: &ir::Variable,
) {
    match stmnt {
        ir::Statement::SaveVariable { var } if var == previous => {
            *var = n_var.clone();
        }
        ir::Statement::SaveVariable { .. } => {}
        ir::Statement::Assignment { value, .. } => {
            match value {
                ir::Value::Expression(exp) => {
                    match exp {
                        ir::Expression::BinaryOp { left, right, .. } => {
                            replace_operand(left, previous, n_var);
                            replace_operand(right, previous, n_var);
                        }
                        ir::Expression::Cast { base, .. } => {
                            replace_operand(base, previous, n_var);
                        }
                        ir::Expression::UnaryOp { base, .. } => {
                            replace_operand(base, previous, n_var);
                        }
                        other => {
                            dbg!(other);
                            todo!()
                        }
                    };
                }
                ir::Value::Variable(var) if var == previous => {
                    *var = n_var.clone();
                }
                ir::Value::Variable(_) => {}
                ir::Value::Phi { sources } => {
                    for src in sources.iter_mut() {
                        if &src.var == previous {
                            src.var = n_var.clone();
                        }
                    }

                    let first = sources.first().expect("");
                    if sources.iter().all(|v| first.var == v.var) {
                        let var = first.var.clone();

                        *value = ir::Value::Variable(var);
                    }
                }
                ir::Value::Constant(_) => {}
                ir::Value::Unknown => {}
            };
        }
        ir::Statement::Return(Some(var)) if var == previous => {
            *var = n_var.clone();
        }
        ir::Statement::Return(_) => {}
        ir::Statement::Jump(_, _) => {}
        ir::Statement::JumpTrue(var, _, _) if var == previous => {
            *var = n_var.clone();
        }
        ir::Statement::JumpTrue(_, _, _) => {}
        ir::Statement::SaveGlobalVariable { var } if var == previous => {
            *var = n_var.clone();
        }
        ir::Statement::SaveGlobalVariable { .. } => {}
        other => {
            dbg!(&other);
            todo!()
        }
    };
}

fn reconstruct_ssa(block: &ir::BasicBlock, reloads: ReloadList) {
    let reloads: Vec<_> = reloads
        .into_iter()
        .flat_map(|(b, vars)| vars.into_iter().map(move |v| (b.clone(), v)))
        .collect();
    let reloaded_vars: HashSet<_> = reloads
        .iter()
        .map(|(_, r)| r.previous.name.clone())
        .collect();

    dbg!(reloads
        .iter()
        .map(|(_, r)| r.clone())
        .collect::<Vec<Reload>>());

    // Iterate over the current block and all the blocks that can be reached from it
    for tmp_b in block.block_iter() {
        let preds = tmp_b.get_predecessors();
        let mut search_statements = tmp_b.get_statements();
        let mut statements = tmp_b.get_statements();

        let mut index = 0;
        while let Some(stmnt) = statements.get_mut(index) {
            let s_vars = stmnt.used_vars();
            let s_re_vars = s_vars
                .into_iter()
                .filter(|v| reloaded_vars.contains(&v.name));

            for re_var in s_re_vars {
                let stmnt = statements.get_mut(index).unwrap();

                let prev_def = find_previous_definition(
                    &preds,
                    search_statements.iter().take(index).cloned(),
                    &re_var.name,
                );
                match prev_def {
                    PrevDefinition::Single(n_var) => {
                        replace_used_variables(stmnt, &re_var, &n_var);
                    }
                    PrevDefinition::Mutliple(vars) => {
                        let n_var = vars.get(0).unwrap().0.next_gen();
                        dbg!(&n_var);

                        let n_var_assign = ir::Statement::Assignment {
                            target: n_var.clone(),
                            value: ir::Value::Phi {
                                sources: vars
                                    .iter()
                                    .map(|(var, block)| ir::PhiEntry {
                                        var: var.clone(),
                                        block: block.weak_ptr(),
                                    })
                                    .collect(),
                            },
                        };

                        replace_used_variables(stmnt, &re_var, &n_var);

                        statements.insert(index, n_var_assign);
                        search_statements = statements.clone();

                        index += 1;
                    }
                };
            }

            index += 1;
        }

        tmp_b.set_statements(statements);
    }
}

#[derive(Debug, Clone, Default)]
struct BlockSpillingData {
    entry_vars: BTreeSet<ir::Variable>,
    entry_spilled: BTreeSet<ir::Variable>,
    exit_vars: BTreeSet<ir::Variable>,
    exit_spilled: BTreeSet<ir::Variable>,
}

fn connect_preds<'i, PI>(
    pred_iter: PI,
    pred_data: &BTreeMap<*const InnerBlock, BlockSpillingData>,
    entry_vars: &BTreeSet<ir::Variable>,
    entry_spilled: &BTreeSet<ir::Variable>,
    reloads: &mut ReloadList,
) where
    PI: IntoIterator<Item = &'i WeakBlockPtr>,
{
    for pred in pred_iter {
        let pred_block = pred.upgrade().unwrap();
        let p_data = match pred_data.get(&pred.as_ptr()) {
            Some(d) => d,
            None => {
                continue;
            }
        };

        let to_reload = entry_vars.difference(&p_data.exit_vars);
        let to_spill = (entry_spilled.difference(&p_data.exit_spilled))
            .filter(|v| p_data.exit_vars.contains(v));

        let pred_statements = pred_block.get_statements();

        let pred_jump_index = pred_statements
            .iter()
            .enumerate()
            .find(|(_, stmnt)| match stmnt {
                ir::Statement::Jump(target, _) => target.as_ptr() == pred.as_ptr(),
                ir::Statement::JumpTrue(_, target, _) => target.as_ptr() == pred.as_ptr(),
                _ => false,
            })
            .map(|(i, _)| i);
        let pred_reloads: Vec<Reload> = to_reload
            .into_iter()
            .map(|r_var| Reload {
                previous: r_var.clone(),
                var: r_var.next_gen(),
                position: pred_jump_index.unwrap_or_else(|| {
                    // This case should probably never really be hit, but it is in certain test cases so its fine for now, I think
                    dbg!("Predecessor must be global", pred_block.as_ptr());
                    pred_statements.len()
                }),
            })
            .collect();

        reloads.add(pred_block, pred_reloads);

        for s_var in to_spill {
            todo!("Spill Variable: {:?}", s_var);
        }
    }
}

fn intialize_register_sets(
    func: &ir::FunctionDefinition,
    available_registers: usize,
    max_vars: usize,
    dbg_ctx: &mut DebugContext,
) {
    let root = &func.block;
    let mut result: HashMap<_, BlockSpillingData> = root
        .get_predecessors()
        .into_iter()
        .map(|p| p.as_ptr())
        .zip(std::iter::repeat(BlockSpillingData::default()))
        .collect();
    assert!(result.len() == 1);

    let mut reloads = ReloadList::new();
    let mut pending_blocks = vec![root.clone()];

    let mut update_blocks = Vec::new();

    loop {
        let (current, needs_update) = match pending_blocks.iter().find(|b| {
            b.get_predecessors()
                .into_iter()
                .all(|p| result.contains_key(&p.as_ptr()))
        }) {
            Some(c) => (c, false),
            None if !pending_blocks.is_empty() => {
                let best_fit = pending_blocks
                    .iter()
                    .max_by(|x, y| {
                        x.get_predecessors()
                            .into_iter()
                            .filter(|p| result.contains_key(&p.as_ptr()))
                            .count()
                            .cmp(
                                &y.get_predecessors()
                                    .into_iter()
                                    .filter(|p| result.contains_key(&p.as_ptr()))
                                    .count(),
                            )
                    })
                    .expect("There are pending Blocks so we should find a maximum even if they would have the same value");

                (best_fit, true)
            }
            None => break,
        };

        let preds = current.get_predecessors();
        let succs = current.successors();

        let pred_data: BTreeMap<_, _> = preds
            .iter()
            .filter_map(|p| {
                let ptr = p.as_ptr();
                let pred_vars = result.get(&ptr).cloned()?;

                Some((ptr, pred_vars))
            })
            .collect();

        let entry_vars = if succs.len() == 2 && preds.len() == 2 {
            let live_in_from_pred: BTreeSet<ir::Variable> = preds
                .iter()
                .filter_map(|p| {
                    let data = result.get(&p.as_ptr())?;
                    Some(data.exit_vars.clone())
                })
                .flatten()
                .collect();
            let live_in_from_phis: BTreeSet<ir::Variable> = current
                .get_statements()
                .into_iter()
                .filter_map(|s| match s {
                    ir::Statement::Assignment {
                        target,
                        value: ir::Value::Phi { .. },
                    } => Some(target),
                    _ => None,
                })
                .collect();
            let i_b = {
                let mut tmp = live_in_from_pred;
                tmp.extend(live_in_from_phis);
                tmp
            };

            let vars_used_in_loop = {
                let loop_block = if succs
                    .iter()
                    .next()
                    .unwrap()
                    .1
                    .predecessor_iter()
                    .any(|p| p.as_ptr() == current.as_ptr())
                {
                    succs.iter().next().unwrap().1.clone()
                } else {
                    succs.iter().nth(1).unwrap().1.clone()
                };

                let mut result = BTreeSet::new();
                let mut pending = vec![loop_block];
                while let Some(pend) = pending.pop() {
                    result.extend(pend.get_statements().into_iter().flat_map(|s| {
                        let mut tmp = s.used_vars();
                        if let ir::Statement::Assignment { target, .. } = s {
                            tmp.push(target);
                        }

                        tmp
                    }));

                    pending.extend(
                        pend.successors()
                            .into_iter()
                            .filter(|(_, s)| s.as_ptr() != current.as_ptr())
                            .map(|(_, s)| s),
                    );
                }

                result
            };

            let candidates: BTreeSet<_> = i_b.intersection(&vars_used_in_loop).cloned().collect();

            assert!(candidates.len() <= max_vars);

            let mut entry_vars = candidates;
            // Fill entry_vars with more candidates to increase efficiency
            // TODO
            // Use different metric for refill-count, instead use max_vars - max-used-registers-in-loop

            // This should be the Max-Number of used registers in the Loop itself
            let max_used_registers = max_vars;
            let fill_count = max_vars.saturating_sub(max_used_registers);
            for (_, ent) in (0..fill_count).zip(i_b) {
                entry_vars.insert(ent);
            }

            entry_vars
        } else {
            // Section 4.2 for normal Blocks
            let all = pred_data.values().cloned().map(|d| d.exit_vars).fold(
                pred_data.values().next().cloned().unwrap().exit_vars,
                |acc, current| acc.intersection(&current).cloned().collect(),
            );

            let some = pred_data
                .values()
                .cloned()
                .flat_map(|d| d.exit_vars)
                .filter(|v| !all.contains(v))
                .fold(BTreeSet::new(), |mut acc, current| {
                    acc.insert(current);
                    acc
                });

            let mut entry_vars = all;
            // Fill entry_vars with more variables for better efficiency
            for (_, ent) in (0..(max_vars.saturating_sub(entry_vars.len()))).zip(some) {
                entry_vars.insert(ent);
            }

            entry_vars
        };

        // Section 4.3
        let entry_spilled: BTreeSet<ir::Variable> = (pred_data
            .values()
            .cloned()
            .map(|d| d.exit_spilled)
            .fold(BTreeSet::new(), |mut acc, mut current| {
                acc.append(&mut current);
                acc
            }))
        .intersection(&entry_vars)
        .cloned()
        .collect();

        connect_preds(
            preds.iter(),
            &pred_data,
            &entry_vars,
            &entry_spilled,
            &mut reloads,
        );

        if needs_update {
            update_blocks.push(current.clone());
        }

        let mut exit_vars = entry_vars.clone();
        let mut exit_spilled = entry_spilled.clone();

        let mut next_use_visited = HashSet::new();
        let next_use_distance = current
            .successors()
            .into_values()
            .map(|b| next_use_distances(&b, &mut next_use_visited))
            .fold(HashMap::new(), |mut acc, elem| {
                merge_distances(&mut acc, elem.into_iter());
                acc
            });
        let min_reloads = min_algorithm(
            current,
            &mut exit_vars,
            &mut exit_spilled,
            max_vars,
            next_use_distance,
        );

        reloads.add(current.clone(), min_reloads);

        result.insert(
            current.as_ptr(),
            BlockSpillingData {
                entry_vars,
                entry_spilled,
                exit_vars,
                exit_spilled,
            },
        );

        let succ_blocks = current.successors().into_iter().map(|(_, b)| b);
        pending_blocks.extend(succ_blocks);
        pending_blocks.retain(|b| !result.contains_key(&b.as_ptr()));
    }

    for update_block in update_blocks {
        let prev_spill_data = result.get(&update_block.as_ptr()).unwrap();

        let preds = update_block.get_predecessors();
        let pred_data: BTreeMap<_, _> = preds
            .iter()
            .filter_map(|p| {
                let ptr = p.as_ptr();
                let pred_vars = result.get(&ptr).cloned()?;

                Some((ptr, pred_vars))
            })
            .collect();

        connect_preds(
            preds.iter(),
            &pred_data,
            &prev_spill_data.entry_vars,
            &prev_spill_data.entry_spilled,
            &mut reloads,
        );
    }

    reconstruct_ssa(root, reloads);

    assert!(pending_blocks.is_empty());
}

fn limit(
    current_vars: &mut BTreeSet<ir::Variable>,
    spilled: &mut BTreeSet<ir::Variable>,
    instructions: &[ir::Statement],
    current: usize,
    max_vars: usize,
    across_distance: &HashMap<ir::Variable, usize>,
) -> Vec<ir::Variable> {
    let local_distance: BTreeMap<_, _> = instructions
        .iter()
        .skip(current)
        .enumerate()
        .rev()
        .flat_map(|(i, s)| s.used_vars().into_iter().zip(std::iter::repeat(i)))
        .collect();

    let max_local_distance = local_distance.values().cloned().max().unwrap_or(0);
    let max_across_distance = across_distance
        .values()
        .cloned()
        .map(|v| v + max_local_distance)
        .max()
        .unwrap_or(0)
        .max(local_distance.values().cloned().max().unwrap_or(0));

    let mut sorted_current = current_vars
        .iter()
        .cloned()
        .map(|var| match local_distance.get(&var) {
            Some(dist) => (var, *dist),
            None => match across_distance.get(&var) {
                Some(ad) => (var, *ad + max_local_distance),
                None => (var, max_across_distance + 3),
            },
        })
        .collect::<Vec<_>>();
    sorted_current.sort_by_key(|(_, d)| *d);

    let mut result = Vec::new();
    for (tmp, dist) in sorted_current.iter().skip(max_vars) {
        if !spilled.contains(tmp) && *dist < max_across_distance + 2 {
            result.push(tmp.clone());
        }

        spilled.remove(tmp);
    }

    *current_vars = sorted_current
        .into_iter()
        .take(max_vars)
        .map(|(v, _)| v)
        .collect();
    result
}

pub struct RegisterConfig {
    pub general_purpose_count: usize,
    pub floating_point_count: usize,
}

pub fn spill(
    func: &ir::FunctionDefinition,
    available_registers: RegisterConfig,
    dbg_ctx: &mut DebugContext,
) {
    // TODO
    // Handle the max register Count correctly
    intialize_register_sets(
        func,
        available_registers.general_purpose_count,
        available_registers.general_purpose_count,
        dbg_ctx,
    );
}
