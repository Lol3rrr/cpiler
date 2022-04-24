//! Contains an implementation for a DirectedGraph

use std::{collections::HashMap, fmt::Debug, hash::Hash};

mod successor;

mod chain;
pub use chain::DirectedChain;
mod flatchain;
pub use flatchain::DirectedFlatChain;

mod mocks;

/// A simple Directed Graph Datastructure
pub struct DirectedGraph<N>
where
    N: GraphNode,
{
    // The initial/starting Node
    initial: Option<N::Id>,
    // All the Nodes in the Graph
    nodes: HashMap<N::Id, N>,
}

/// Describes a generic GraphNode as used by a DirectedGraph
pub trait GraphNode {
    /// The Iterator over the Successors of a Node
    type SuccessorIterator: Iterator<Item = Self::Id>;
    /// The ID of a single Node
    type Id: Eq + Hash + Clone + Copy + Debug;

    /// The ID of the current Node
    fn id(&self) -> Self::Id;
    /// An Iterator over the Successors of this Node
    fn successors(&self) -> Self::SuccessorIterator;
}

/// This trait should be implemented to allow for Visualizing the Graph
pub trait GraphVisual
where
    Self: GraphNode,
{
    /// The Name that should be used to display the Node in a Visualization
    fn name(&self) -> String {
        format!("node_{:?}", self.id())
    }

    /// The Attributes that should be added to the Node for visualization
    fn node_attributes(&self) -> Option<HashMap<String, String>> {
        None
    }
    /// The Attributes that should be added to the Edge from the current Node to the given
    /// Sucessor
    fn edge_attributes(&self, _succ: Self::Id) -> Option<HashMap<String, String>> {
        None
    }
}

impl<N> DirectedGraph<N>
where
    N: GraphNode,
{
    /// Creates a new empty Graph Instance
    pub fn new() -> Self {
        Self {
            initial: None,
            nodes: HashMap::new(),
        }
    }

    /// Adds a new Node to the Graph and returns it's Id, if this is the first Node it will also be
    /// set as the initial/starting Node of the Graph
    pub fn add_node(&mut self, node: N) -> N::Id {
        let id = node.id();

        self.nodes.insert(id, node);
        if self.initial.is_none() {
            self.initial = Some(id);
        }

        id
    }

    /// Gets a shared reference to the Node with the given Id
    pub fn get_node(&self, id: &N::Id) -> Option<&N> {
        self.nodes.get(id)
    }

    /// Gets a mutable reference to the Node with the given Id
    pub fn get_mut_node(&mut self, id: &N::Id) -> Option<&mut N> {
        self.nodes.get_mut(id)
    }

    /// Removes the Node with the given Id from the Graph
    pub fn remove_node(&mut self, id: N::Id) -> Option<N> {
        self.nodes.remove(&id)
    }

    /// Obtains an Iterator like Object to allow iterating over Chains of Entries in the Graph
    pub fn chain_iter<'g, 'c>(&'g self) -> DirectedChain<'c, N>
    where
        'g: 'c,
    {
        DirectedChain::new(self, self.initial)
    }

    /// Returns a Chain like [`chain_iter`], which starts at the Node with the given id
    pub(crate) fn chain_from<'g, 'c>(&'g self, id: N::Id) -> DirectedChain<'c, N>
    where
        'g: 'c,
    {
        DirectedChain::new(self, Some(id))
    }
}

impl<N> DirectedGraph<N>
where
    N: GraphNode + GraphVisual,
{
    /// Used to generate a Graphviz representation of the Graph itself
    pub fn to_graphviz(&self) -> String {
        let mut result = "digraph D {".to_string();

        for (_, node) in self.nodes.iter() {
            let node_name = node.name();

            let mut attribute_str = String::new();
            if let Some(attributes) = node.node_attributes() {
                for (attr_name, attr_value) in attributes {
                    let line = format!("{}={}", attr_name, attr_value);
                    attribute_str.push_str(&line);
                }
            }

            let node_line = format!("{}[{}];", node_name, attribute_str);
            result.push_str(&node_line);

            for succ in node.successors().map(|id| self.nodes.get(&id).unwrap()) {
                let mut attribute_str = String::new();
                if let Some(attributes) = node.edge_attributes(succ.id()) {
                    for (attr_name, attr_value) in attributes {
                        let line = format!("{}={}", attr_name, attr_value);
                        attribute_str.push_str(&line);
                    }
                }

                let line = format!("{} -> {} [{}];", node_name, succ.name(), attribute_str);
                result.push_str(&line);
            }
        }

        result.push('}');
        result
    }
}

impl<N> Default for DirectedGraph<N>
where
    N: GraphNode,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'g, N> From<&'g DirectedGraph<N>> for DirectedChain<'g, N>
where
    N: GraphNode,
{
    fn from(other: &'g DirectedGraph<N>) -> Self {
        other.chain_iter()
    }
}

/// An Entry as returned by a Chain
pub enum ChainEntry<'n, N>
where
    N: GraphNode,
{
    /// A Single Node
    Node(&'n N),
    /// The Control-Flow has split into multiple Branches, contains a Vec of all the Chains to go over
    /// all the Entries in their respective Chains
    Branched {
        /// The two Sides of the Branch
        sides: (DirectedChain<'n, N>, Option<DirectedChain<'n, N>>),
    },
    /// The Graph has encountered a Cycle, contains the Chain to go over all the Entries in the
    /// Body of the Cycle
    Cycle {
        /// The Head of the Cycle, i.e. the last shared Node between the inner Chain and the Rest of
        /// the Graph
        head: N::Id,
        /// The Chain for the Body of the Cycle, without the Head, so starting with the Node just
        /// after the Head and ending with the last Node before jumping back to the Head
        inner: DirectedChain<'n, N>,
    },
}

impl<'g, N> Debug for ChainEntry<'g, N>
where
    N: GraphNode,
    N::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(n) => {
                write!(f, "Node({:?})", n.id())
            }
            Self::Branched { sides } => f.debug_struct("Branched").field("sides", sides).finish(),
            Self::Cycle { head, inner } => f
                .debug_struct("Cycle")
                .field("head", head)
                .field("inner", inner)
                .finish(),
        }
    }
}

impl<'g, N> ChainEntry<'g, N>
where
    N: GraphNode,
{
    /// Checks if the Entry is a Node
    pub fn is_node(&self) -> bool {
        matches!(self, Self::Node { .. })
    }
    /// Checks if the Entry is a Branch
    pub fn is_branch(&self) -> bool {
        matches!(self, Self::Branched { .. })
    }
    /// Checks if the Entry is a Cycle
    pub fn is_cycle(&self) -> bool {
        matches!(self, Self::Cycle { .. })
    }
}

enum InnerChained {
    First,
    Second,
    Done,
}

/// Used to Chain 2 chains together
pub struct DirectedChainedChain<'g1, 'g2, N>
where
    N: GraphNode,
{
    state: InnerChained,
    first: DirectedChain<'g1, N>,
    second: DirectedChain<'g2, N>,
}

impl<'g1, 'g2, N> DirectedChainedChain<'g1, 'g2, N>
where
    N: GraphNode,
{
    /// Creates a new Chain from the given Chains
    pub fn new(first: DirectedChain<'g1, N>, second: DirectedChain<'g2, N>) -> Self {
        Self {
            state: InnerChained::First,
            first,
            second,
        }
    }

    /// Gets the next Entry
    pub fn next_entry(&mut self) -> Option<ChainEntry<'_, N>> {
        match self.state {
            InnerChained::First => {
                if let Some(e) = self.first.next_entry() {
                    return Some(e);
                }

                self.state = InnerChained::Second;

                if let Some(e) = self.second.next_entry() {
                    return Some(e);
                }

                None
            }
            InnerChained::Second => {
                if let Some(e) = self.second.next_entry() {
                    return Some(e);
                }

                self.state = InnerChained::Done;

                None
            }
            InnerChained::Done => None,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::directed::successor::{succ_type, SuccType};

    use super::{mocks::MockNode, *};

    #[test]
    fn new() {
        let _ = DirectedGraph::<mocks::MockNode>::new();
    }

    #[test]
    fn single_node_stuff() {
        let mut graph = DirectedGraph::new();

        let id = graph.add_node(mocks::MockNode {
            id: 0,
            successors: vec![],
        });

        assert!(graph.get_node(&id).is_some());
        assert!(graph.get_mut_node(&id).is_some());

        let rem = graph.remove_node(id);
        assert!(rem.is_some());

        assert!(graph.get_node(&id).is_none());
        assert!(graph.get_mut_node(&id).is_none());
    }

    #[test]
    fn linear_graph() {
        let mut graph = DirectedGraph::new();

        graph.add_node(mocks::MockNode {
            id: 0,
            successors: vec![1],
        });
        graph.add_node(mocks::MockNode {
            id: 1,
            successors: vec![],
        });

        let result = succ_type(
            graph.get_node(&0).unwrap(),
            &graph,
            successor::Context::None,
        );
        assert_eq!(Some(SuccType::Single(1)), result);

        let mut chain = graph.chain_iter();
        {
            let entry = chain.next_entry();
            assert!(entry.is_some());
            assert!(matches!(entry.unwrap(), ChainEntry::Node(n) if n.id() == 0));
        }
        {
            let entry = chain.next_entry();
            assert!(entry.is_some());
            assert!(matches!(entry.unwrap(), ChainEntry::Node(n) if n.id() == 1));
        }
        {
            let entry = chain.next_entry();
            assert!(entry.is_none());
        }

        let mut flat_chain = graph.chain_iter().flatten();
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_none());
        }
    }

    #[test]
    fn branched_graph() {
        let mut graph = DirectedGraph::new();

        graph.add_node(mocks::MockNode {
            id: 0,
            successors: vec![1, 2],
        });
        graph.add_node(mocks::MockNode {
            id: 1,
            successors: vec![3],
        });
        graph.add_node(mocks::MockNode {
            id: 2,
            successors: vec![3],
        });
        graph.add_node(mocks::MockNode {
            id: 3,
            successors: vec![],
        });

        let result = succ_type(
            graph.get_node(&0).unwrap(),
            &graph,
            successor::Context::None,
        );
        assert_eq!(
            Some(SuccType::Branched {
                sides: (1, Some(2)),
                end: 3,
            }),
            result
        );

        let mut chain = graph.chain_iter();
        {
            let entry = chain.next_entry();
            assert!(entry.is_some());
            assert!(matches!(entry.unwrap(), ChainEntry::Node(n) if n.id() == 0));
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            let (mut left, right) = match entry {
                ChainEntry::Branched { sides } => (sides.0, sides.1),
                _ => {
                    panic!("Expected a Branched Entry");
                }
            };

            let mut right = right.unwrap();

            {
                let raw_entry = left.next_entry();
                assert!(raw_entry.is_some());
                assert!(matches!(raw_entry.unwrap(), ChainEntry::Node(n) if n.id() == 1));

                let raw_entry = left.next_entry();
                assert!(raw_entry.is_none());
            }
            {
                let raw_entry = right.next_entry();
                assert!(raw_entry.is_some());
                assert!(matches!(raw_entry.unwrap(), ChainEntry::Node(n) if n.id() == 2));

                let raw_entry = right.next_entry();
                assert!(raw_entry.is_none());
            }
        }
        {
            let entry = chain.next_entry();
            assert!(entry.is_some());
            assert!(matches!(entry.unwrap(), ChainEntry::Node(n) if n.id() == 3));
        }

        let mut flat_chain = graph.chain_iter().flatten();
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_none());
        }
    }

    #[test]
    fn branched_only_left() {
        let mut graph = DirectedGraph::new();

        graph.add_node(mocks::MockNode {
            id: 0,
            successors: vec![1, 2],
        });
        graph.add_node(mocks::MockNode {
            id: 1,
            successors: vec![2],
        });
        graph.add_node(mocks::MockNode {
            id: 2,
            successors: vec![],
        });

        let mut root_chain = graph.chain_iter();

        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 0));
        }
        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Branched { .. }));

            let (mut left, right) = match entry {
                ChainEntry::Branched { sides, .. } => sides,
                _ => unreachable!(),
            };

            assert!(right.is_none());

            {
                let raw_entry = left.next_entry();
                assert!(raw_entry.is_some());
            }
            {
                let raw_entry = left.next_entry();
                assert!(raw_entry.is_none());
            }
        }
        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 2));
        }
        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_none());
        }
    }

    #[test]
    fn branched_only_right() {
        let mut graph = DirectedGraph::new();

        graph.add_node(mocks::MockNode {
            id: 0,
            successors: vec![2, 1],
        });
        graph.add_node(mocks::MockNode {
            id: 1,
            successors: vec![2],
        });
        graph.add_node(mocks::MockNode {
            id: 2,
            successors: vec![],
        });

        let mut root_chain = graph.chain_iter();

        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 0));
        }
        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            dbg!(&entry);
            assert!(matches!(entry, ChainEntry::Branched { .. }));

            let (mut left, right) = match entry {
                ChainEntry::Branched { sides, .. } => sides,
                _ => unreachable!(),
            };

            assert!(right.is_none());

            {
                let raw_entry = left.next_entry();
                assert!(raw_entry.is_some());
            }
            {
                let raw_entry = left.next_entry();
                assert!(raw_entry.is_none());
            }
        }
        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 2));
        }
        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_none());
        }
    }

    #[test]
    fn cycle_graph_first_cycle() {
        let mut graph = DirectedGraph::new();

        graph.add_node(mocks::MockNode {
            id: 0,
            successors: vec![1, 2],
        });
        graph.add_node(mocks::MockNode {
            id: 1,
            successors: vec![0],
        });
        graph.add_node(mocks::MockNode {
            id: 2,
            successors: vec![],
        });

        let result = succ_type(
            graph.get_node(&0).unwrap(),
            &graph,
            successor::Context::None,
        );
        assert_eq!(
            Some(SuccType::Cycle {
                inner: 1,
                following: Some(2)
            }),
            result
        );

        let mut chain = graph.chain_iter();

        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 0));
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Cycle { .. }));

            let mut inner = match entry {
                ChainEntry::Cycle { inner, .. } => inner,
                _ => unreachable!("We previously asserted this"),
            };

            {
                let raw_entry = inner.next_entry();
                assert!(raw_entry.is_some());
                let entry = raw_entry.unwrap();
                assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 1))
            }
            {
                let raw_entry = inner.next_entry();
                assert!(raw_entry.is_none());
            }
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 2));
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_none());
        }

        let mut flat_chain = graph.chain_iter().flatten();
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_none());
        }
    }
    #[test]
    fn cycle_graph_second_cycle() {
        let mut graph = DirectedGraph::new();

        graph.add_node(mocks::MockNode {
            id: 0,
            successors: vec![1, 2],
        });
        graph.add_node(mocks::MockNode {
            id: 1,
            successors: vec![],
        });
        graph.add_node(mocks::MockNode {
            id: 2,
            successors: vec![0],
        });

        let result = succ_type(
            graph.get_node(&0).unwrap(),
            &graph,
            successor::Context::None,
        );
        assert_eq!(
            Some(SuccType::Cycle {
                inner: 2,
                following: Some(1)
            }),
            result
        );

        let mut chain = graph.chain_iter();

        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 0));
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Cycle { .. }));

            let mut inner = match entry {
                ChainEntry::Cycle { inner, .. } => inner,
                _ => unreachable!("We just asserted this"),
            };

            {
                let raw_entry = inner.next_entry();
                assert!(raw_entry.is_some());
                let entry = raw_entry.unwrap();
                assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 2));
            }
            {
                let raw_entry = inner.next_entry();
                assert!(raw_entry.is_none());
            }
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 1));
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_none());
        }
    }

    #[test]
    fn nested_cycle() {
        let mut graph = DirectedGraph::new();

        graph.add_node(mocks::MockNode {
            id: 0,
            successors: vec![1, 3],
        });
        graph.add_node(mocks::MockNode {
            id: 1,
            successors: vec![0, 2],
        });
        graph.add_node(mocks::MockNode {
            id: 2,
            successors: vec![1],
        });
        graph.add_node(mocks::MockNode {
            id: 3,
            successors: vec![],
        });

        let mut outer_chain = graph.chain_iter();

        {
            println!("First Node");
            let raw_entry = outer_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 0));
        }
        {
            println!("Outer Chain");
            let raw_entry = outer_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Cycle { .. }));

            let mut first_inner = match entry {
                ChainEntry::Cycle { inner, .. } => inner,
                _ => unreachable!(""),
            };

            {
                println!("Outer Chain - First Node");
                let raw_entry = first_inner.next_entry();
                assert!(raw_entry.is_some());
                let entry = raw_entry.unwrap();
                assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 1));
            }
            {
                println!("Outer Chain - Inner Chain");
                let raw_entry = first_inner.next_entry();
                assert!(raw_entry.is_some());
                dbg!(&raw_entry);
                let entry = raw_entry.unwrap();
                assert!(matches!(entry, ChainEntry::Cycle { .. }));

                let mut second_inner = match entry {
                    ChainEntry::Cycle { inner, .. } => inner,
                    _ => unreachable!(""),
                };

                {
                    println!("Outer Chain - Inner Chain - First Node");
                    let raw_entry = second_inner.next_entry();
                    assert!(raw_entry.is_some());
                }
                {
                    let raw_entry = second_inner.next_entry();
                    dbg!(&raw_entry);
                    assert!(raw_entry.is_none());
                }
            }
            {
                let raw_entry = first_inner.next_entry();
                dbg!(&raw_entry);
                assert!(raw_entry.is_none());
            }
        }
        {
            let raw_entry = outer_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 3));
        }
        {
            let raw_entry = outer_chain.next_entry();
            assert!(raw_entry.is_none());
        }

        let mut flat_chain = graph.chain_iter().flatten();
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_none());
        }
    }

    #[test]
    fn branch_in_cycle() {
        let mut graph = DirectedGraph::new();

        graph.add_node(mocks::MockNode {
            id: 0,
            successors: vec![1, 5],
        });
        graph.add_node(mocks::MockNode {
            id: 1,
            successors: vec![2, 3],
        });
        graph.add_node(mocks::MockNode {
            id: 2,
            successors: vec![4],
        });
        graph.add_node(mocks::MockNode {
            id: 3,
            successors: vec![4],
        });
        graph.add_node(mocks::MockNode {
            id: 4,
            successors: vec![0],
        });
        graph.add_node(mocks::MockNode {
            id: 5,
            successors: vec![],
        });

        let mut root_chain = graph.chain_iter();

        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 0));
        }
        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Cycle { .. }));

            let mut inner_chain = match entry {
                ChainEntry::Cycle { inner, .. } => inner,
                _ => unreachable!(""),
            };

            {
                let raw_entry = inner_chain.next_entry();
                assert!(raw_entry.is_some());
                let entry = raw_entry.unwrap();
                assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 1));
            }
            {
                let raw_entry = inner_chain.next_entry();
                assert!(raw_entry.is_some());
                let entry = raw_entry.unwrap();
                assert!(matches!(entry, ChainEntry::Branched { .. }));

                let (mut left, right) = match entry {
                    ChainEntry::Branched { sides, .. } => sides,
                    _ => unreachable!(),
                };

                let mut right = right.unwrap();

                {
                    let raw_entry = left.next_entry();
                    assert!(raw_entry.is_some());
                    let entry = raw_entry.unwrap();
                    assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 2));
                }
                {
                    let raw_entry = left.next_entry();
                    assert!(raw_entry.is_none());
                }

                {
                    let raw_entry = right.next_entry();
                    assert!(raw_entry.is_some());
                    let entry = raw_entry.unwrap();
                    assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 3));
                }
                {
                    let raw_entry = right.next_entry();
                    assert!(raw_entry.is_none());
                }
            }
            {
                let raw_entry = inner_chain.next_entry();
                assert!(raw_entry.is_some());
                let entry = raw_entry.unwrap();
                assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 4));
            }
            {
                let raw_entry = inner_chain.next_entry();
                assert!(raw_entry.is_none());
            }
        }
        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 5));
        }
        {
            let raw_entry = root_chain.next_entry();
            assert!(raw_entry.is_none());
        }

        let mut flat_chain = graph.chain_iter().flatten();
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_some());
        }
        {
            let entry = flat_chain.next_entry();
            assert!(entry.is_none());
        }
    }

    #[test]
    fn cycle_with_conditional_break() {
        let mut graph = DirectedGraph::new();

        graph.add_node(MockNode {
            id: 0,
            successors: vec![1],
        });
        // Loop Head
        graph.add_node(MockNode {
            id: 1,
            successors: vec![2, 10],
        });
        // After Loop
        graph.add_node(MockNode {
            id: 10,
            successors: vec![],
        });
        // Branch Head
        graph.add_node(MockNode {
            id: 2,
            successors: vec![3, 4],
        });
        // After Branch
        graph.add_node(MockNode {
            id: 3,
            successors: vec![1],
        });
        // Inner Branched side with Break
        graph.add_node(MockNode {
            id: 4,
            successors: vec![3, 10],
        });

        let mut chain = graph.chain_iter();
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 0));
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 1));
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            let mut inner_chain = match entry {
                ChainEntry::Cycle { inner, head } => {
                    assert_eq!(head, 1);
                    inner
                }
                _ => unreachable!(),
            };

            {
                let raw_entry = inner_chain.next_entry();
                assert!(raw_entry.is_some());
                let entry = raw_entry.unwrap();
                assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 2));
            }
            {
                let raw_entry = inner_chain.next_entry();
                assert!(raw_entry.is_some());
                let entry = raw_entry.unwrap();
                let (mut left, right) = match entry {
                    ChainEntry::Branched { sides } => sides,
                    _ => unreachable!(),
                };

                assert!(right.is_none());

                {
                    let raw_entry = left.next_entry();
                    assert!(raw_entry.is_some());
                }
                {
                    let raw_entry = left.next_entry();
                    dbg!(&raw_entry);
                    assert!(raw_entry.is_none());
                }
            }
            {
                let raw_entry = inner_chain.next_entry();
                assert!(raw_entry.is_some());
                let entry = raw_entry.unwrap();
                assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 3));
            }
            {
                let raw_entry = inner_chain.next_entry();
                assert!(raw_entry.is_none());
            }
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
            let entry = raw_entry.unwrap();
            assert!(matches!(entry, ChainEntry::Node(n) if n.id() == 10));
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_none());
        }

        let flat_chain = graph.chain_iter().flatten();
        for b in flat_chain {
            dbg!(b.id());
        }

        // panic!("")
    }
}
