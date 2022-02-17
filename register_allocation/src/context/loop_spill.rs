use std::collections::HashSet;

pub enum SpillResult {
    Outer { var: ir::Variable },
}

fn spill_outer(
    largest_vars: &HashSet<ir::Variable>,
    header: ir::BasicBlock,
    outer: ir::BasicBlock,
) -> Option<ir::Variable> {
    let mut used = HashSet::new();

    let mut visited: HashSet<_> = vec![outer.as_ptr()].into_iter().collect();
    let mut to_visit = vec![header];

    while let Some(next) = to_visit.pop() {
        if visited.contains(&next.as_ptr()) {
            continue;
        }
        visited.insert(next.as_ptr());

        used.extend(next.get_statements().into_iter().flat_map(|s| {
            let mut tmp = s.used_vars();
            if let ir::Statement::Assignment { target, .. } = s {
                tmp.push(target);
            }
            tmp
        }));

        to_visit.extend(next.successors().into_iter().map(|(_, b)| b));
    }

    let mut diff: Vec<_> = largest_vars.difference(&used).cloned().collect();

    diff.pop()
}

pub fn spill_var(
    largest_vars: HashSet<ir::Variable>,
    block: ir::BasicBlock,
    start_index: usize,
    header: ir::BasicBlock,
    inner: ir::BasicBlock,
    outer: ir::BasicBlock,
) -> SpillResult {
    if let Some(outer_var) = spill_outer(&largest_vars, header, outer) {
        dbg!(&outer_var);
        todo!("Spill Variable from outisde the Loop")
    }

    todo!("Spill Variable from inside the Loop")
}
