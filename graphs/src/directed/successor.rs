use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use super::{DirectedGraph, GraphNode};

#[derive(Debug, PartialEq)]
pub enum SuccType<I> {
    Single(I),
    Branched { sides: (I, Option<I>), end: I },
    Cycle { inner: I, following: Option<I> },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Context<I> {
    None,
    OuterGraph { head: I, following: Option<I> },
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

    debug_assert!(start_node.successors().count() <= 2);

    let first = successors.next()?;

    let second = match successors.next() {
        Some(s) => s,
        None => return Some(SuccType::Single(first)),
    };

    dbg!(first, second, ctx);

    #[allow(clippy::single_match)]
    match ctx {
        Context::OuterGraph {
            following: Some(following),
            ..
        } => {
            if first == following {
                return Some(SuccType::Single(second));
            }
            if second == following {
                return Some(SuccType::Single(first));
            }
        }
        _ => {}
    };

    /*
    match (is_end(&first), is_end(&second)) {
        (true, true) => return None,
        (true, false) => return Some(SuccType::Single(second)),
        (false, true) => return Some(SuccType::Single(first)),
        (false, false) => {}
    };
    */

    let mut first_ids: HashMap<N::Id, usize> = HashMap::new();
    {
        let mut remaining = vec![first];
        let mut index = 0;
        while let Some(id) = remaining.pop() {
            let tmp = graph.get_node(&id).unwrap();

            first_ids.insert(id, index);
            index += 1;
            remaining.extend(
                tmp.successors()
                    .filter(|i| !first_ids.contains_key(i))
                    .filter(|i| match ctx {
                        Context::None => true,
                        Context::OuterGraph { head, .. } => *i != head,
                    }),
            );
        }
    }

    {
        let mut remaining = vec![start];
        let mut visited = HashSet::new();
        visited.insert(first);
        let mut jumped_to_head = false;
        while let Some(id) = remaining.pop() {
            if first_ids.contains_key(&id) {
                if id == start {
                    match ctx {
                        Context::None => {
                            return Some(SuccType::Cycle {
                                inner: first,
                                following: Some(second),
                            });
                        }
                        Context::OuterGraph { head, .. } => {
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

                if first == id {
                    println!("1");
                    return Some(SuccType::Branched {
                        sides: (second, None),
                        end: id,
                    });
                } else if second == id {
                    println!("2");
                    return Some(SuccType::Branched {
                        sides: (first, None),
                        end: id,
                    });
                }

                debug_assert_ne!(first, id);
                debug_assert_ne!(second, id);

                println!("3");
                return Some(SuccType::Branched {
                    sides: (first, Some(second)),
                    end: id,
                });
            }

            let tmp = graph.get_node(&id).unwrap();
            visited.insert(id);

            jumped_to_head |= tmp.successors().any(|s| start == s);

            remaining.extend(tmp.successors().filter(|i| !visited.contains(i)).filter(
                |i| match ctx {
                    Context::None => true,
                    Context::OuterGraph { head, .. } => *i != head,
                },
            ));
        }

        if visited.contains(&start) && jumped_to_head {
            match ctx {
                Context::None => {
                    return Some(SuccType::Cycle {
                        inner: second,
                        following: Some(first),
                    });
                }
                Context::OuterGraph { head, .. } => {
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

        if start_node.successors().any(|s| s == second) {
            Some(SuccType::Branched {
                sides: (second, None),
                end: first,
            })
        } else {
            todo!("Second is following")
        }
    }
}
