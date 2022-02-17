use std::collections::{HashMap, HashSet};

use ir::Variable;

pub struct SpillResult {
    pub var: ir::Variable,
    pub save_index: usize,
    pub load_block: ir::BasicBlock,
    pub load_index: usize,
}

#[derive(Debug)]
struct VarDistance {
    pub total: usize,
    pub in_block: usize,
}

fn get_offsets<SI>(
    statements: SI,
    start_offset: usize,
    unknown: &mut HashSet<ir::Variable>,
) -> (usize, HashMap<ir::Variable, VarDistance>)
where
    SI: Iterator<Item = ir::Statement>,
{
    let mut offsets = HashMap::new();
    let mut count = 0;
    let iter = statements
        .enumerate()
        .map(|(index, stmnt)| (index + start_offset, stmnt, index))
        .inspect(|_| {
            count += 1;
        });

    for (distance, stmnt, index) in iter {
        let used = stmnt.used_vars();
        let used_unknowns: Vec<_> = used.into_iter().filter(|v| unknown.contains(v)).collect();

        for u_v in used_unknowns {
            unknown.remove(&u_v);
            offsets.insert(
                u_v,
                VarDistance {
                    total: distance,
                    in_block: index,
                },
            );
        }
    }

    let end_distance = start_offset + count;
    (end_distance, offsets)
}

fn calc_var_offsets(
    start: ir::BasicBlock,
    start_index: usize,
    mut unknowns: HashSet<ir::Variable>,
    end_block: ir::BasicBlock,
) -> HashMap<ir::Variable, (VarDistance, ir::BasicBlock)> {
    let start_stmnts = start.get_statements().into_iter().skip(start_index);
    let (mut distance, initial_offsets) = get_offsets(start_stmnts, 0, &mut unknowns);

    let mut offsets: HashMap<ir::Variable, (VarDistance, ir::BasicBlock)> = initial_offsets
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                VarDistance {
                    total: v.total,
                    in_block: v.in_block + start_index,
                },
            )
        })
        .map(|(k, v)| (k, (v, start.clone())))
        .collect();

    let mut visited: HashSet<_> = vec![start.as_ptr()].into_iter().collect();
    let mut block = start;

    while !unknowns.is_empty() {
        let mut succs: Vec<_> = block
            .successors()
            .into_iter()
            .filter(|(ptr, _)| visited.contains(ptr))
            .collect();

        if succs.is_empty() {
            break;
        }

        if succs.len() == 1 {
            let (_, tmp) = succs.remove(0);

            if tmp.as_ptr() == end_block.as_ptr() {
                break;
            }

            let tmp_statements = tmp.get_statements();
            let (n_distance, tmp_offsets) =
                get_offsets(tmp_statements.into_iter(), distance, &mut unknowns);

            distance = n_distance;
            for n_offset in tmp_offsets {
                if !offsets.contains_key(&n_offset.0) {
                    offsets.insert(n_offset.0, (n_offset.1, tmp.clone()));
                }
            }

            visited.insert(tmp.as_ptr());

            block = tmp;
            continue;
        }

        todo!("Offset for linear Section with 2 Successors");
    }

    offsets
}

pub fn determine_spill_var(
    vars: HashSet<Variable>,
    start_block: ir::BasicBlock,
    start_index: usize,
    end_block: ir::BasicBlock,
) -> SpillResult {
    let offsets = calc_var_offsets(start_block, start_index, vars, end_block);

    let largest_distance = offsets
        .into_iter()
        .max_by_key(|(_, (distance, _))| distance.total)
        .unwrap();

    let (var, (distance, block)) = largest_distance;

    SpillResult {
        var,
        save_index: start_index,
        load_block: block,
        load_index: distance.in_block,
    }
}
