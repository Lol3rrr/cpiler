use std::collections::HashSet;

use super::{DirectedGraph, GraphNode};

#[derive(Debug, PartialEq)]
pub enum SuccType<I> {
    Single(I),
    Branched { sides: (I, I), end: I },
    Cycle { inner: I, following: I },
}

/// Used to determine the Type of Successors
/// * Single
/// * Branched
/// * Cycle
pub fn succ_type<N>(
    start_node: &N,
    graph: &DirectedGraph<N>,
    end: Option<N::Id>,
) -> Option<SuccType<N::Id>>
where
    N: GraphNode,
{
    let start = start_node.id();
    let mut successors = start_node.successors();

    let mut visited: HashSet<N::Id> = HashSet::new();
    visited.insert(start);
    if let Some(end) = end {
        visited.insert(end);
    }

    let first = successors.next()?;

    let second = match successors.next() {
        Some(s) => s,
        None => return Some(SuccType::Single(first)),
    };

    assert!(successors.next().is_none());

    let mut first_ids: HashSet<N::Id> = HashSet::new();
    {
        let mut remaining = vec![first];
        while let Some(id) = remaining.pop() {
            let tmp = graph.get_node(&id).unwrap();

            first_ids.insert(id);
            remaining.extend(
                tmp.successors()
                    .filter(|i| !first_ids.contains(i))
                    .filter(|i| Some(*i) != end),
            );
        }
    }

    {
        let mut remaining = vec![start];
        let mut visited = HashSet::new();
        visited.insert(first);
        while let Some(id) = remaining.pop() {
            if first_ids.contains(&id) {
                if id == start {
                    return Some(SuccType::Cycle {
                        inner: first,
                        following: second,
                    });
                }

                return Some(SuccType::Branched {
                    sides: (first, second),
                    end: id,
                });
            }

            let tmp = graph.get_node(&id).unwrap();
            visited.insert(id);

            remaining.extend(tmp.successors().filter(|i| !visited.contains(i)));
        }

        if visited.contains(&start) {
            return Some(SuccType::Cycle {
                inner: second,
                following: first,
            });
        }
    }

    unreachable!()
}
