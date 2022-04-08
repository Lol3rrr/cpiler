use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::RegisterConfig;

// Returns a closure that returns true as long as it still has space for the given Variable based on the given RegisterConfig
fn register_skips(regs: RegisterConfig) -> impl FnMut(&ir::Variable) -> bool {
    let mut gps_used = 0;
    let mut floats_used = 0;
    move |v| {
        if v.ty.is_float() {
            floats_used += 1;
            floats_used <= regs.floating_point_count
        } else {
            gps_used += 1;
            gps_used <= regs.general_purpose_count
        }
    }
}

/// Returns the Collection of Variables that are spilled
pub fn limit(
    // The currently live Variables
    current_vars: &mut BTreeSet<ir::Variable>,
    // The currently spilled Variables
    spilled: &mut BTreeSet<ir::Variable>,
    // The list of instructions we are currently considering for our liveness analysis
    instructions: &[ir::Statement],
    // The maximum Number of Registers available
    max_vars: RegisterConfig,
    across_distance: &HashMap<ir::Variable, usize>,
) -> Vec<ir::Variable> {
    let local_distance: BTreeMap<_, _> = instructions
        .iter()
        .enumerate()
        .rev()
        .flat_map(|(i, s)| s.used_vars().into_iter().zip(std::iter::repeat(i)))
        .collect();

    // The maximum distance found in the current Block, this is mainly used as an offset added to the Distances across Blocks
    let max_local_distance = local_distance.values().cloned().max().unwrap_or(0);
    // The maximum distance found across Blocks
    let max_across_distance = across_distance
        .values()
        .cloned()
        .map(|v| v + max_local_distance)
        .max()
        .unwrap_or(0)
        .max(local_distance.values().cloned().max().unwrap_or(0));

    // Sorts the current Variables by their next use distance, in either the current Block or the Distance from the Next-Block
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

    let mut result = Vec::with_capacity(sorted_current.len());
    let mut sorted_skip_closure = register_skips(max_vars);
    for (tmp, dist) in sorted_current
        .iter()
        .filter(|(v, _)| !(sorted_skip_closure(v)))
    {
        // The Variable has not been spilled before but is still used afterwards (either in the current Block or anohter one)
        if !spilled.contains(tmp) && *dist < max_across_distance + 2 {
            // The Variable should be spilled
            result.push(tmp.clone());
        }

        // Remove the Variable from the previously spilled variables
        spilled.remove(tmp);
    }

    // Assign the new currently live Variables to the current_vars variable
    {
        let mut skip_fn = register_skips(max_vars);

        *current_vars = sorted_current
            .into_iter()
            .filter_map(|(v, _)| skip_fn(&v).then(|| v))
            .collect();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_closure_1() {
        let max = RegisterConfig {
            general_purpose_count: 2,
            floating_point_count: 1,
        };

        let mut closure = register_skips(max);

        assert!(closure(&ir::Variable::tmp(0, ir::Type::I32)));
        assert!(closure(&ir::Variable::tmp(0, ir::Type::I32)));
        assert!(!(closure(&ir::Variable::tmp(0, ir::Type::I32))));
        assert!(closure(&ir::Variable::tmp(0, ir::Type::Float)));
        assert!(!(closure(&ir::Variable::tmp(0, ir::Type::Float))));
    }
}
