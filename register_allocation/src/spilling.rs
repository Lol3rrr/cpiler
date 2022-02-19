//! This is based on this Paper https://link.springer.com/content/pdf/10.1007%252F978-3-642-00722-4_13.pdf

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::spill;

fn next_use_distances(root: &ir::BasicBlock) -> HashMap<ir::Variable, Option<usize>> {
    let mut block_distance: HashMap<_, _> = vec![(root.as_ptr(), 0)].into_iter().collect();

    let in_block_distances: HashMap<_, _> = root
        .block_iter()
        .map(|block| {
            let var_distances: HashMap<_, _> = block
                .get_statements()
                .into_iter()
                .enumerate()
                .rev()
                .flat_map(|(i, stmnt)| stmnt.used_vars().into_iter().zip(std::iter::repeat(i + 1)))
                .collect();
            (block.as_ptr(), var_distances)
        })
        .collect();

    dbg!(&in_block_distances);

    todo!("Calculate next use distances")
}

#[derive(Debug)]
struct Reload {
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

fn reconstruct_ssa(block: &ir::BasicBlock, reloads: Vec<(ir::BasicBlock, Vec<Reload>)>) {
    let reloads = reloads
        .into_iter()
        .flat_map(|(b, vars)| vars.into_iter().map(move |v| (b.clone(), v)));

    for (r_block, r_reload) in reloads {
        dbg!(r_block.as_ptr(), &r_reload);

        let mut statements = r_block.get_statements();
        for stmnt in statements.iter_mut().skip(r_reload.position + 1) {
            match stmnt {
                ir::Statement::SaveVariable { var } if var == &r_reload.previous => {
                    *var = r_reload.var.clone();
                }
                ir::Statement::SaveVariable { .. } => {}
                ir::Statement::Assignment { value, .. } => {
                    match value {
                        ir::Value::Expression(exp) => {
                            match exp {
                                ir::Expression::BinaryOp { left, right, .. } => {
                                    replace_operand(left, &r_reload.previous, &r_reload.var);
                                    replace_operand(right, &r_reload.previous, &r_reload.var);
                                }
                                other => {
                                    dbg!(other);
                                    todo!()
                                }
                            };
                        }
                        ir::Value::Unknown => {}
                        other => {
                            dbg!(&other);
                            todo!()
                        }
                    };
                }
                ir::Statement::Return(Some(var)) if var == &r_reload.previous => {
                    *var = r_reload.var.clone();
                }
                ir::Statement::Return(_) => {}
                other => {
                    dbg!(&other);
                    todo!()
                }
            };
        }
        r_block.set_statements(statements);

        let succs = r_block.successors();
        assert!(succs.is_empty());
    }
}

#[derive(Debug, Clone, Default)]
struct BlockSpillingData {
    entry_vars: BTreeSet<ir::Variable>,
    entry_spilled: BTreeSet<ir::Variable>,
    exit_vars: BTreeSet<ir::Variable>,
    exit_spilled: BTreeSet<ir::Variable>,
}

fn intialize_register_sets(root: &ir::BasicBlock, available_registers: usize, max_vars: usize) {
    let mut result: HashMap<_, BlockSpillingData> = root
        .get_predecessors()
        .into_iter()
        .map(|p| p.as_ptr())
        .zip(std::iter::repeat(BlockSpillingData::default()))
        .collect();
    assert!(result.len() == 1);

    let mut reloads = Vec::new();
    let mut pending_blocks = vec![root.clone()];
    loop {
        let current = match pending_blocks.iter().enumerate().find(|(_, b)| {
            b.get_predecessors()
                .into_iter()
                .all(|p| result.contains_key(&p.as_ptr()))
        }) {
            Some((i, _)) => pending_blocks.remove(i),
            None => break,
        };

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

            let some = pred_data.values().cloned().map(|d| d.exit_vars).fold(
                BTreeSet::new(),
                |mut acc, mut current| {
                    acc.append(&mut current);
                    acc
                },
            );

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
        let tmp_reloads = min_algorithm(&current, &mut exit_vars, &mut exit_spilled, max_vars);
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
    }

    reconstruct_ssa(root, reloads);

    assert!(pending_blocks.is_empty());
}

#[derive(Debug)]
enum MinAction {
    Spill,
    Reload { n_var: ir::Variable },
}

fn min_algorithm(
    block: &ir::BasicBlock,
    current_vars: &mut BTreeSet<ir::Variable>,
    spilled: &mut BTreeSet<ir::Variable>,
    max_vars: usize,
) -> Vec<Reload> {
    let statements = block.get_statements();

    let mut spills = Vec::new();
    let mut reloads = Vec::new();

    for (index, stmnt) in statements.iter().enumerate() {
        let used_vars = stmnt.used_vars();
        let r: BTreeSet<ir::Variable> = used_vars
            .into_iter()
            .filter(|v| !current_vars.contains(&v))
            .collect();

        //dbg!(&r, &current_vars);
        for tmp_use in r.iter() {
            current_vars.insert(tmp_use.clone());
            spilled.insert(tmp_use.clone());
        }

        let definition = match &stmnt {
            ir::Statement::Assignment { target, .. } => Some(target.clone()),
            _ => None,
        };

        let spill_first = limit(current_vars, spilled, &statements, index, max_vars);
        let spill_second = limit(
            current_vars,
            spilled,
            &statements,
            index + 1,
            max_vars - definition.as_ref().map(|_| 1).unwrap_or(0),
        );

        for spill_var in spill_first.into_iter().chain(spill_second) {
            spills.push((index, spill_var));
        }

        if let Some(defed) = definition {
            current_vars.insert(defed);
        }

        for r_var in r {
            reloads.push((index, r_var));
        }
    }

    let mut n_statements = statements;
    let mut action_iter: Vec<_> = spills
        .into_iter()
        .map(|(i, v)| (i, v, MinAction::Spill))
        .chain(reloads.clone().into_iter().map(|(i, v)| {
            (
                i,
                v.clone(),
                MinAction::Reload {
                    n_var: v.next_gen(),
                },
            )
        }))
        .collect();
    action_iter.sort_by_key(|(i, _, _)| *i);
    for (offset, (index, var, action)) in action_iter.iter().enumerate() {
        match action {
            MinAction::Spill => {
                n_statements.insert(index + offset, spill::save_statement(var.clone()));
            }
            MinAction::Reload { n_var } => {
                n_statements.insert(index + offset, spill::load_statement(n_var.clone()));
            }
        };
    }

    block.set_statements(n_statements);

    action_iter
        .into_iter()
        .enumerate()
        .filter_map(|(o, (i, v, action))| match action {
            MinAction::Spill => None,
            MinAction::Reload { n_var } => Some((o, (i, v, n_var))),
        })
        .map(|(offset, (index, var, n_var))| Reload {
            var: n_var,
            previous: var,
            position: offset + index,
        })
        .collect()
}

fn limit(
    current_vars: &mut BTreeSet<ir::Variable>,
    spilled: &mut BTreeSet<ir::Variable>,
    instructions: &[ir::Statement],
    current: usize,
    max_vars: usize,
) -> Vec<ir::Variable> {
    let distance: BTreeMap<_, _> = instructions
        .into_iter()
        .skip(current)
        .enumerate()
        .rev()
        .flat_map(|(i, s)| s.used_vars().into_iter().zip(std::iter::repeat(i)))
        .collect();

    let mut sorted_current = current_vars
        .iter()
        .cloned()
        .map(|var| match distance.get(&var) {
            Some(dist) => (var, *dist),
            None => (var, instructions.len() + 1),
        })
        .collect::<Vec<_>>();
    sorted_current.sort_by_key(|(_, d)| *d);

    let mut result = Vec::new();
    for (tmp, dist) in sorted_current.iter().skip(max_vars) {
        if !spilled.contains(tmp) && *dist < instructions.len() {
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

pub fn spill(root: ir::BasicBlock, available_registers: usize) {
    //let n_use_distance = next_use_distances(&root);
    //dbg!(&n_use_distance);

    let register_sets = intialize_register_sets(&root, available_registers, available_registers);
}
