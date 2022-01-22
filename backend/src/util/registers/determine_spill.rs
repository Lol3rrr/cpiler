use std::collections::HashSet;

use ir::Variable;

fn get_offsets<SI>(
    statements: SI,
    start_offset: usize,
    unknown: &mut HashSet<ir::Variable>,
    offsets: &mut Vec<(ir::Variable, usize)>,
) -> usize
where
    SI: Iterator<Item = ir::Statement>,
{
    let mut count = 0;
    let iter = statements
        .enumerate()
        .map(|(index, stmnt)| (index + start_offset, stmnt))
        .inspect(|_| {
            count += 1;
        });

    for (distance, stmnt) in iter {
        let used = stmnt.used_vars();
        let used_unknowns: Vec<_> = used.into_iter().filter(|v| unknown.contains(v)).collect();

        for u_v in used_unknowns {
            unknown.remove(&u_v);
            offsets.push((u_v, distance));
        }
    }

    start_offset + count
}

pub fn determine_spill_var(
    vars: HashSet<Variable>,
    start_block: ir::BasicBlock,
    start_index: usize,
) -> ir::Variable {
    let mut unknown = vars;

    let statements = start_block.get_statements().into_iter().skip(start_index);
    let mut block = start_block;
    let mut offsets = Vec::new();

    let mut offset = get_offsets(statements, 0, &mut unknown, &mut offsets);

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
            block.get_statements().into_iter(),
            offset,
            &mut unknown,
            &mut offsets,
        );
        visited.insert(block.as_ptr());
    }

    let first = offsets
        .into_iter()
        .max_by(|(_, a_d), (_, b_d)| a_d.cmp(b_d))
        .unwrap();

    first.0
}
