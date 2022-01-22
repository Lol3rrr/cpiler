use std::collections::HashSet;

pub enum SpillResult {
    /// The Variable to spill is not in the Conditional itself, so you can save it before starting
    /// the Conditional and then load it again afterwards
    OuterVariable {
        /// The Variable to spill
        var: ir::Variable,
    },
}

fn inner_used(start: ir::BasicBlock, end: &ir::BasicBlock) -> HashSet<ir::Variable> {
    let mut result = HashSet::new();
    let mut visited = HashSet::new();
    let mut to_visit: Vec<_> = vec![start];

    while let Some(current) = to_visit.pop() {
        if current.as_ptr() == end.as_ptr() {
            continue;
        }
        if visited.contains(&current.as_ptr()) {
            continue;
        }
        visited.insert(current.as_ptr());

        let c_statements = current.get_statements();
        let used: HashSet<_> = c_statements
            .into_iter()
            .flat_map(|s| {
                let mut base = s.used_vars();
                if let ir::Statement::Assignment { target, .. } = s {
                    base.push(target);
                }
                base
            })
            .collect();

        result.extend(used);

        let succs = current.successors();
        to_visit.extend(succs.into_iter().map(|(_, b)| b));
    }

    result
}

fn spill_outer(
    largest_vars: &HashSet<ir::Variable>,
    current_block: ir::BasicBlock,
    other_block: Option<ir::BasicBlock>,
    end: ir::BasicBlock,
) -> Option<ir::Variable> {
    let mut used = HashSet::new();

    let first_used = inner_used(current_block, &end);
    used.extend(first_used);

    if let Some(other_block) = other_block {
        let second_used = inner_used(other_block, &end);
        used.extend(second_used);
    }

    let diff: HashSet<_> = largest_vars.difference(&used).cloned().collect();

    if diff.is_empty() {
        return None;
    }

    // TODO
    // Use a better heuristic for determining the outer Variable to spill, but for now this should
    // work, but may not provide the best possible solution

    diff.into_iter().next()
}

pub fn spill_var(
    largest_vars: &HashSet<ir::Variable>,
    largest_block: ir::BasicBlock,
    start_index: usize,
    header: ir::BasicBlock,
    end: ir::BasicBlock,
    current_start: ir::BasicBlock,
    other_start: Option<ir::BasicBlock>,
) -> SpillResult {
    if let Some(outer_var) = spill_outer(
        largest_vars,
        current_start.clone(),
        other_start.clone(),
        end.clone(),
    ) {
        return SpillResult::OuterVariable { var: outer_var };
    }

    todo!("Spill in Conditional")
}
