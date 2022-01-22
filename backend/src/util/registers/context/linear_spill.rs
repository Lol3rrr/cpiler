use std::collections::HashSet;

use ir::Variable;

pub struct SpillResult {
    pub var: ir::Variable,
    pub save_index: usize,
    pub load_block: ir::BasicBlock,
    pub load_index: usize,
}

fn get_offsets<SI>(
    block: ir::BasicBlock,
    statements: SI,
    start_offset: usize,
    unknown: &mut HashSet<ir::Variable>,
    offsets: &mut Vec<(ir::Variable, usize, ir::BasicBlock, usize)>,
) -> usize
where
    SI: Iterator<Item = ir::Statement>,
{
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
            offsets.push((u_v, distance, block.clone(), index));
        }
    }

    start_offset + count
}

pub fn determine_spill_var(
    vars: HashSet<Variable>,
    start_block: ir::BasicBlock,
    start_index: usize,
) -> SpillResult {
    let mut unknown = vars;

    let statements = start_block.get_statements().into_iter().skip(start_index);
    let mut block = start_block;
    let mut offsets = Vec::new();

    let mut offset = get_offsets(block.clone(), statements, 0, &mut unknown, &mut offsets);

    let mut visited = HashSet::new();
    visited.insert(block.as_ptr());

    while !unknown.is_empty() {
        let mut succs: Vec<_> = block
            .successors()
            .into_iter()
            .filter(|(ptr, _)| visited.contains(ptr))
            .collect();
        if succs.is_empty() {
            break;
        }

        if succs.len() > 1 {
            break;
        }

        let (_, single_succ) = succs.remove(0);
        block = single_succ;

        offset = get_offsets(
            block.clone(),
            block.get_statements().into_iter(),
            offset,
            &mut unknown,
            &mut offsets,
        );
        visited.insert(block.as_ptr());
    }

    let first = offsets
        .into_iter()
        .max_by(|(_, a_d, _, _), (_, b_d, _, _)| a_d.cmp(b_d))
        .unwrap();

    SpillResult {
        var: first.0,
        save_index: start_index,
        load_block: first.2,
        load_index: first.3,
    }
}
