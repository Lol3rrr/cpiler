//! This is based on this Paper https://link.springer.com/content/pdf/10.1007%252F978-3-642-00722-4_13.pdf

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    ops::Sub,
    vec,
};

use crate::debug_ctx::DebugContext;

mod min;
use graphs::directed::{ChainEntry, DirectedGraph};
use ir::{InnerBlock, WeakBlockPtr};
use min::min_algorithm;

mod reload_list;
use self::reload_list::ReloadList;

mod limit;
use limit::limit;

mod loop_max_pressure;

mod replace;
use replace::replace_used_variables;

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

#[derive(Debug)]
enum PrevDefinition {
    None,
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
            let pred = match weak_pred.upgrade() {
                Some(p) => p,
                None => return PrevDefinition::None,
            };

            let n_preds = pred.get_predecessors();
            let statements = pred.get_statements();

            let pred_def =
                find_previous_definition(&n_preds, statements.into_iter().rev(), var_name);

            if let PrevDefinition::Single(_) = &pred_def {
                return pred_def;
            }

            // dbg!(&pred_def);

            dbg!(var_name);

            todo!("Find Variable in Single Predecessor");
        }
        core::cmp::Ordering::Greater => {
            let found_vars = preds
                .iter()
                .flat_map(|raw_pred| {
                    let pred = raw_pred.upgrade().unwrap();

                    let pred_preds = pred.get_predecessors();

                    let pred_var = find_previous_definition(
                        &pred_preds,
                        pred.get_statements().into_iter().rev(),
                        var_name,
                    );

                    match pred_var {
                        PrevDefinition::None => vec![],
                        PrevDefinition::Single(var) => vec![(var, pred)],
                        PrevDefinition::Mutliple(vars) => vars,
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

/// Brings the given Function back into SSA form, which may not have been preserved during spilling
fn reconstruct_ssa(graph: DirectedGraph<ir::BasicBlock>, reloads: ReloadList) {
    let reloads: Vec<_> = reloads
        .into_iter()
        .flat_map(|(b, vars)| vars.into_iter().map(move |v| (b.clone(), v)))
        .collect();
    let reloaded_vars: HashSet<_> = reloads
        .iter()
        .map(|(_, r)| r.previous.name.clone())
        .collect();

    for tmp_b in graph.chain_iter().flatten() {
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
                    PrevDefinition::None => {
                        todo!()
                    }
                    PrevDefinition::Single(n_var) => {
                        replace_used_variables(stmnt, &re_var, &n_var);
                    }
                    PrevDefinition::Mutliple(vars) => {
                        let n_var = vars.get(0).unwrap().0.next_gen();

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
                    // TODO
                    // Figure out what to do here
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

fn init_reg_set_chain(
    chain: graphs::directed::DirectedChain<'_, ir::BasicBlock>,
    max_vars: RegisterConfig,
    spill_data: &mut HashMap<*const InnerBlock, BlockSpillingData>,
    reloads: &mut ReloadList,
    update_blocks: &mut Vec<ir::BasicBlock>,
) {
    let mut peek_chain = chain.peekable();

    while let Some(entry) = peek_chain.next() {
        let (entry_vars, pred_data, preds, current) = match (entry, peek_chain.peek()) {
            (ChainEntry::Node(node), Some(ChainEntry::Node(_)))
            | (ChainEntry::Node(node), Some(ChainEntry::Branched { .. }))
            | (ChainEntry::Node(node), None) => {
                let preds = node.get_predecessors();

                // Get the Data for the Predecessors that have already been processed
                let pred_data: BTreeMap<_, _> = preds
                    .iter()
                    .filter_map(|p| {
                        let ptr = p.as_ptr();
                        let pred_vars = spill_data.get(&ptr).cloned()?;

                        Some((ptr, pred_vars))
                    })
                    .collect();

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
                for (_, ent) in (0..(max_vars
                    .general_purpose_count
                    .saturating_sub(entry_vars.len())))
                    .zip(some)
                {
                    entry_vars.insert(ent);
                }

                (entry_vars, pred_data, preds, node)
            }
            (ChainEntry::Node(node), Some(ChainEntry::Cycle { inner, .. })) => {
                update_blocks.push(node.clone());

                let preds = node.get_predecessors();

                // Get the Data for the Predecessors that have already been processed
                let pred_data: BTreeMap<_, _> = preds
                    .iter()
                    .filter_map(|p| {
                        let ptr = p.as_ptr();
                        let pred_vars = spill_data.get(&ptr).cloned()?;

                        Some((ptr, pred_vars))
                    })
                    .collect();

                let live_in_from_pred: BTreeSet<ir::Variable> = preds
                    .iter()
                    .filter_map(|p| {
                        let data = spill_data.get(&p.as_ptr())?;
                        Some(data.exit_vars.clone())
                    })
                    .flatten()
                    .collect();
                let live_in_from_phis: BTreeSet<ir::Variable> = node
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

                let vars_used_in_loop: BTreeSet<ir::Variable> = inner
                    .duplicate()
                    .flatten()
                    .flat_map(|b| b.get_statements())
                    .flat_map(|s| s.used_vars())
                    .collect();

                // The Candidates are the highest Priority to NOT spill
                let candidates: BTreeSet<_> =
                    i_b.intersection(&vars_used_in_loop).cloned().collect();

                assert!(candidates.len() <= max_vars.total());

                let mut entry_vars = candidates;

                // Fill entry_vars with more candidates to increase efficiency
                /*
                let max_used_registers =
                loop_max_pressure::max_pressure(node.clone(), inner.duplicate(), |var| {
                    inner
                        .duplicate()
                        .flatten()
                        .skip(1)
                        .flat_map(|b| b.get_statements())
                        .flat_map(|s| s.used_vars())
                        .any(|v| &v == var)
                });
                */
                //dbg!(max_used_registers);

                // The number of still available Registers for other Variables to avoid spilling them
                //
                // This is currently not used because for some reason it does not really work as intended
                // and needs more investigation on why that is the case
                //let mut fill_count = max_vars - max_used_registers;

                let mut fill_count = RegisterConfig {
                    general_purpose_count: 0,
                    floating_point_count: 0,
                };

                // This closure will return true for Variables as long as they will fit into the "Budget"
                // defined using the fill_count variable
                let filter_closure = |v: &ir::Variable| {
                    if v.ty.is_float() {
                        let prev = fill_count.floating_point_count;
                        fill_count.floating_point_count = prev.saturating_sub(1);
                        prev > 0
                    } else {
                        let prev = fill_count.general_purpose_count;
                        fill_count.general_purpose_count = prev.saturating_sub(1);
                        prev > 0
                    }
                };
                for ent in i_b.into_iter().filter(filter_closure) {
                    entry_vars.insert(ent);
                }

                (entry_vars, pred_data, preds, node)
            }
            (ChainEntry::Branched { .. }, _) => {
                unreachable!("Found Branched");
            }
            (ChainEntry::Cycle { .. }, _) => {
                unreachable!("Found Cycle")
            }
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
            reloads,
        );

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

        spill_data.insert(
            current.as_ptr(),
            BlockSpillingData {
                entry_vars,
                entry_spilled,
                exit_vars,
                exit_spilled,
            },
        );

        match peek_chain.peek() {
            Some(ChainEntry::Node(_)) => {}
            Some(ChainEntry::Branched {
                sides: (left, right),
            }) => {
                init_reg_set_chain(
                    left.duplicate(),
                    max_vars,
                    spill_data,
                    reloads,
                    update_blocks,
                );

                if let Some(right) = right {
                    init_reg_set_chain(
                        right.duplicate(),
                        max_vars,
                        spill_data,
                        reloads,
                        update_blocks,
                    );
                }

                let _ = peek_chain.next();
            }
            Some(ChainEntry::Cycle { inner, .. }) => {
                init_reg_set_chain(
                    inner.duplicate(),
                    max_vars,
                    spill_data,
                    reloads,
                    update_blocks,
                );

                let _ = peek_chain.next();
            }
            None => {}
        };
    }
}

// TODO
// Switch to using the Graph
fn intialize_register_sets(
    func: &ir::FunctionDefinition,
    max_vars: RegisterConfig,
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

    // A List of all Reloads that need to be performed
    let mut reloads = ReloadList::new();

    let mut update_blocks = Vec::new();

    let graph = func.to_directed_graph();
    init_reg_set_chain(
        graph.chain_iter(),
        max_vars,
        &mut result,
        &mut reloads,
        &mut update_blocks,
    );

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

    reconstruct_ssa(func.to_directed_graph(), reloads);
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterConfig {
    pub general_purpose_count: usize,
    pub floating_point_count: usize,
}

impl Sub for RegisterConfig {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            general_purpose_count: self
                .general_purpose_count
                .saturating_sub(other.general_purpose_count),
            floating_point_count: self
                .floating_point_count
                .saturating_sub(other.floating_point_count),
        }
    }
}

impl RegisterConfig {
    pub fn total(&self) -> usize {
        self.general_purpose_count + self.floating_point_count
    }
}

pub fn spill(
    func: &ir::FunctionDefinition,
    available_registers: RegisterConfig,
    dbg_ctx: &mut DebugContext,
) {
    // TODO
    // Handle the max register Count correctly
    intialize_register_sets(func, available_registers, dbg_ctx);
}
