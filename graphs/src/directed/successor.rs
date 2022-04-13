use std::{collections::HashSet, fmt::Debug};

use super::{DirectedGraph, GraphNode};

#[derive(Debug, PartialEq)]
pub enum SuccType<I> {
    Single(I),
    Branched { sides: (I, I), end: I },
    Cycle { inner: I, following: Option<I> },
}

#[derive(Debug, Clone, Copy)]
pub enum Context<I> {
    None,
    OuterGraph { head: I },
}

/// Used to determine the Type of Successors
/// * Single
/// * Branched
/// * Cycle
pub fn succ_type<N>(
    start_node: &N,
    graph: &DirectedGraph<N>,
    ctx: Context<N::Id>,
) -> Option<SuccType<N::Id>>
where
    N: GraphNode,
{
    let start = start_node.id();
    let mut successors = start_node.successors();

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
                    .filter(|i| match ctx {
                        Context::None => true,
                        Context::OuterGraph { head } => *i != head,
                    }),
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
                    match ctx {
                        Context::None => {
                            return Some(SuccType::Cycle {
                                inner: first,
                                following: Some(second),
                            });
                        }
                        Context::OuterGraph { head } => {
                            if head == first {
                                return Some(SuccType::Cycle {
                                    inner: second,
                                    following: None,
                                });
                            }

                            return Some(SuccType::Cycle {
                                inner: first,
                                following: Some(second),
                            });
                        }
                    };
                }

                return Some(SuccType::Branched {
                    sides: (first, second),
                    end: id,
                });
            }

            let tmp = graph.get_node(&id).unwrap();
            visited.insert(id);

            remaining.extend(tmp.successors().filter(|i| !visited.contains(i)).filter(
                |i| match ctx {
                    Context::None => true,
                    Context::OuterGraph { head } => *i != head,
                },
            ));
        }

        if visited.contains(&start) {
            match ctx {
                Context::None => {
                    return Some(SuccType::Cycle {
                        inner: second,
                        following: Some(first),
                    });
                }
                Context::OuterGraph { head } => {
                    if head == second {
                        return Some(SuccType::Cycle {
                            inner: first,
                            following: None,
                        });
                    }

                    return Some(SuccType::Cycle {
                        inner: second,
                        following: Some(first),
                    });
                }
            };
        }
    }

    unreachable!()
}
