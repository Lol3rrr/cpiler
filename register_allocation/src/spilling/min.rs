use std::collections::{BTreeSet, HashMap};

use crate::{load_statement, save_statement};

use super::{limit, Reload};

#[derive(Debug)]
enum MinAction {
    Spill,
    Reload { n_var: ir::Variable },
}

pub fn min_algorithm(
    block: &ir::BasicBlock,
    current_vars: &mut BTreeSet<ir::Variable>,
    spilled: &mut BTreeSet<ir::Variable>,
    max_vars: usize,
    across_distance: HashMap<ir::Variable, usize>,
) -> Vec<Reload> {
    let statements = block.get_statements();

    let mut spills = Vec::new();
    let mut reloads = Vec::new();

    for (index, stmnt) in statements.iter().enumerate() {
        let used_vars = stmnt.used_vars();
        let r: BTreeSet<ir::Variable> = used_vars
            .into_iter()
            .filter(|v| !current_vars.contains(v))
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

        let spill_first = limit(
            current_vars,
            spilled,
            &statements,
            index,
            max_vars,
            &across_distance,
        );
        let spill_second = limit(
            current_vars,
            spilled,
            &statements,
            index + 1,
            max_vars - definition.as_ref().map(|_| 1).unwrap_or(0),
            &across_distance,
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
                n_statements.insert(index + offset, save_statement(var.clone()));
            }
            MinAction::Reload { n_var } => {
                n_statements.insert(index + offset, load_statement(n_var.clone()));
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
