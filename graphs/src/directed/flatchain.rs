use super::{ChainEntry, DirectedChain, GraphNode};

/// A Chain that Flattens all control Structures in its root chain and therefore only returns
/// entries
pub struct DirectedFlatChain<'g, N>
where
    N: GraphNode,
{
    /// The Root-Chain which should be flattened
    root_chain: DirectedChain<'g, N>,
    /// Stores the pending Chains that we found but still need to explore
    pending: Vec<DirectedChain<'g, N>>,
}

impl<'g, N> DirectedFlatChain<'g, N>
where
    N: GraphNode,
{
    /// Creates a new Flat-Chain based on the given Chain
    pub fn new(chain: DirectedChain<'g, N>) -> Self {
        Self {
            root_chain: chain,
            pending: Vec::new(),
        }
    }

    fn get_from_pending(&mut self) -> Option<&'g N> {
        while let Some(mut pended) = self.pending.pop() {
            let entry = match pended.next_entry() {
                Some(e) => {
                    self.pending.push(pended);
                    e
                }
                None => continue,
            };

            match entry {
                ChainEntry::Node(n) => return Some(n),
                ChainEntry::Branched {
                    sides: (left, right),
                } => {
                    self.pending.push(left);

                    if let Some(right) = right {
                        self.pending.push(right);
                    }
                }
                ChainEntry::Cycle { inner, .. } => {
                    self.pending.push(inner);
                    continue;
                }
            };
        }

        None
    }

    /// Obtains the next Entry in the Chain
    pub fn next_entry(&mut self) -> Option<&'g N> {
        if let Some(n) = self.get_from_pending() {
            return Some(n);
        }

        loop {
            let raw_next = self.root_chain.next_entry()?;

            match raw_next {
                ChainEntry::Node(n) => return Some(n),
                ChainEntry::Branched {
                    sides: (left, right),
                    ..
                } => {
                    self.pending.push(left);

                    if let Some(right) = right {
                        self.pending.push(right);
                    }

                    if let Some(n) = self.get_from_pending() {
                        return Some(n);
                    }
                }
                ChainEntry::Cycle { inner, .. } => {
                    self.pending.push(inner);

                    if let Some(n) = self.get_from_pending() {
                        return Some(n);
                    }
                }
            }
        }
    }
}

impl<'g, N> Iterator for DirectedFlatChain<'g, N>
where
    N: GraphNode,
{
    type Item = &'g N;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_entry()
    }
}

#[cfg(test)]
mod tests {
    use crate::directed::DirectedGraph;

    use super::super::mocks::*;

    #[test]
    fn linear() {
        let mut graph = DirectedGraph::new();

        graph.add_node(MockNode {
            id: 0,
            successors: vec![1],
        });
        graph.add_node(MockNode {
            id: 1,
            successors: vec![2],
        });
        graph.add_node(MockNode {
            id: 2,
            successors: vec![],
        });

        let mut flat_chain = graph.chain_iter().flatten();

        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert_eq!(
                entry,
                &MockNode {
                    id: 0,
                    successors: vec![1]
                }
            );
        }
        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert_eq!(
                entry,
                &MockNode {
                    id: 1,
                    successors: vec![2]
                }
            );
        }
        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert_eq!(
                entry,
                &MockNode {
                    id: 2,
                    successors: vec![]
                }
            );
        }
        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_none());
        }
    }

    #[test]
    fn branched() {
        let mut graph = DirectedGraph::new();

        graph.add_node(MockNode {
            id: 0,
            successors: vec![1, 2],
        });
        graph.add_node(MockNode {
            id: 1,
            successors: vec![3],
        });
        graph.add_node(MockNode {
            id: 2,
            successors: vec![3],
        });
        graph.add_node(MockNode {
            id: 3,
            successors: vec![],
        });

        let mut flat_chain = graph.chain_iter().flatten();

        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert_eq!(
                entry,
                &MockNode {
                    id: 0,
                    successors: vec![1, 2],
                }
            );
        }
        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert_eq!(
                entry,
                &MockNode {
                    id: 2,
                    successors: vec![3],
                }
            );
        }
        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert_eq!(
                entry,
                &MockNode {
                    id: 1,
                    successors: vec![3],
                }
            );
        }
        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert_eq!(
                entry,
                &MockNode {
                    id: 3,
                    successors: vec![],
                }
            );
        }
        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_none());
        }
    }

    #[test]
    fn cycle() {
        let mut graph = DirectedGraph::new();

        graph.add_node(MockNode {
            id: 0,
            successors: vec![1, 2],
        });
        graph.add_node(MockNode {
            id: 1,
            successors: vec![0],
        });
        graph.add_node(MockNode {
            id: 2,
            successors: vec![],
        });

        let mut flat_chain = graph.chain_iter().flatten();

        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert_eq!(
                entry,
                &MockNode {
                    id: 0,
                    successors: vec![1, 2],
                }
            );
        }
        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert_eq!(
                entry,
                &MockNode {
                    id: 1,
                    successors: vec![0],
                }
            );
        }
        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert_eq!(
                entry,
                &MockNode {
                    id: 2,
                    successors: vec![],
                }
            );
        }
        {
            let raw_entry = flat_chain.next_entry();
            assert!(raw_entry.is_none());
        }
    }
}
