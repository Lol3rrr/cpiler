//! This is based on this Paper https://link.springer.com/content/pdf/10.1007%252F978-3-642-00722-4_13.pdf

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::debug_ctx::DebugContext;

mod min;
use min::min_algorithm;

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

fn next_use_distances(root: &ir::BasicBlock) -> HashMap<ir::Variable, usize> {
    let (root_distances, max_root) = inblock_distance(root);

    let mut distances = root_distances;

    let current = root.clone();
    let succs = current.successors();
    if succs.is_empty() {
    } else if succs.len() == 1 {
        let succ = succs.into_values().next().unwrap();

        let succ_distances = next_use_distances(&succ);

        merge_distances(
            &mut distances,
            succ_distances
                .into_iter()
                .map(|(k, v)| (k, v + max_root + 1)),
        );
    } else if succs.len() == 2 {
        let preds = current.get_predecessors();

        if preds.len() == 1 {
            let succ_distances = succs.into_values().map(|b| next_use_distances(&b));
            for tmp_dist in succ_distances {
                merge_distances(
                    &mut distances,
                    tmp_dist.into_iter().map(|(k, v)| (k, v + max_root + 1)),
                );
            }
        } else {
            todo!("Loop")
        }
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

    if preds.len() == 1 {
        let weak_pred = preds.into_iter().next().unwrap();
        let pred = weak_pred.upgrade().unwrap();

        let n_preds = pred.get_predecessors();
        let statements = pred.get_statements();

        let pred_def = find_previous_definition(&n_preds, statements.into_iter().rev(), var_name);

        if let PrevDefinition::Single(_) = &pred_def {
            return pred_def;
        }

        dbg!(&pred_def);

        todo!("Find Variable in Single Predecessor");
    } else if preds.len() > 1 {
        todo!("Find Variable in Multiple Predecessors")
    }

    todo!()
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
        other => {
            dbg!(&other);
            todo!()
        }
    };
}

fn reconstruct_ssa_statements<'s, SI>(statements: SI, previous: &ir::Variable, n_var: &ir::Variable)
where
    SI: Iterator<Item = &'s mut ir::Statement>,
{
    for stmnt in statements {
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
            other => {
                dbg!(&other);
                todo!()
            }
        };
    }
}

fn reconstruct_ssa(block: &ir::BasicBlock, reloads: Vec<(ir::BasicBlock, Vec<Reload>)>) {
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

    for tmp_b in block.block_iter() {
        let preds = tmp_b.get_predecessors();
        let search_statements = tmp_b.get_statements();
        let mut statements = tmp_b.get_statements();

        for (index, stmnt) in statements.iter_mut().enumerate() {
            let s_vars = stmnt.used_vars();
            let s_re_vars = s_vars
                .into_iter()
                .filter(|v| reloaded_vars.contains(&v.name));
            for re_var in s_re_vars {
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
                        todo!()
                    }
                };
            }
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

    let mut reloads = Vec::new();
    let mut pending_blocks = vec![root.clone()];
    while let Some((_, current)) = pending_blocks.iter().enumerate().find(|(_, b)| {
        b.get_predecessors()
            .into_iter()
            .all(|p| result.contains_key(&p.as_ptr()))
    }) {
        let preds = current.get_predecessors();
        let succs = current.successors();

        let pred_data: BTreeMap<_, _> = preds
            .iter()
            .map(|p| {
                let ptr = p.as_ptr();
                let pred_vars = result.get(&ptr).unwrap();

                (ptr, pred_vars.clone())
            })
            .collect();

        let entry_vars = if succs.len() == 2 && preds.len() == 2 {
            todo!("Loop Header");
        } else {
            // Section 4.2 for normal Blocks
            let all = pred_data.values().cloned().map(|d| d.exit_vars).fold(
                pred_data.values().cloned().next().unwrap().exit_vars,
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

            // TODO
            // Also fill the remaining slots with Variables from some
            let entry_vars = all;

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

        // TODO
        // This does not work for loop headers yet because one of the Predecessor may not have been
        // processed yet
        for pred in preds.iter() {
            let p_data = pred_data.get(&pred.as_ptr()).unwrap();

            let to_reload = entry_vars.difference(&p_data.exit_vars);
            let to_spill = (entry_spilled.difference(&p_data.exit_spilled))
                .filter(|v| p_data.exit_vars.contains(&v));

            for r_var in to_reload {
                todo!()
            }
            for s_var in to_spill {
                todo!()
            }
        }

        let mut exit_vars = entry_vars.clone();
        let mut exit_spilled = entry_spilled.clone();

        let next_use_distance = current
            .successors()
            .into_values()
            .map(|b| next_use_distances(&b))
            .fold(HashMap::new(), |mut acc, elem| {
                merge_distances(&mut acc, elem.into_iter());
                acc
            });
        let tmp_reloads = min_algorithm(
            &current,
            &mut exit_vars,
            &mut exit_spilled,
            max_vars,
            next_use_distance,
        );
        reloads.push((current.clone(), tmp_reloads));

        result.insert(
            current.as_ptr(),
            BlockSpillingData {
                entry_vars,
                entry_spilled,
                exit_vars,
                exit_spilled,
            },
        );

        pending_blocks.extend(current.successors().into_iter().map(|(_, b)| b));
        pending_blocks.retain(|b| !result.contains_key(&b.as_ptr()));

        //dbg_ctx.add_state(&func);
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
    //let n_use_distance = next_use_distances(&root);
    //dbg!(&n_use_distance);

    // TODO
    // Handle the max register Count correctly
    intialize_register_sets(
        func,
        available_registers.general_purpose_count,
        available_registers.general_purpose_count,
        dbg_ctx,
    );
}
