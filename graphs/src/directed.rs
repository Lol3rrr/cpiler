//! Contains an implementation for a DirectedGraph

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

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
    type Id: Eq + Hash + Clone + Copy;

    /// The ID of the current Node
    fn id(&self) -> Self::Id;
    /// An Iterator over the Successors of this Node
    fn successors(&self) -> Self::SuccessorIterator;
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
        DirectedChain {
            previous: None,
            next: self.initial,
            end: None,
            graph: self,
        }
    }

    /// Returns a Chain like [`chain_iter`], which starts at the Node with the given id
    pub fn chain_from<'g, 'c>(&'g self, id: N::Id) -> DirectedChain<'c, N>
    where
        'g: 'c,
    {
        DirectedChain {
            previous: None,
            next: Some(id),
            end: None,
            graph: self,
        }
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

/// A Chain that can be used to Iterate over a Chain of Entries in Graph
pub struct DirectedChain<'g, N>
where
    N: GraphNode,
{
    /// The ID of the last Entry we returned
    previous: Option<N::Id>,
    /// The ID of the next Entry we will return
    next: Option<N::Id>,
    /// The ID up until which  we should return Entries (exclusive)
    end: Option<N::Id>,
    /// The HashMap of Nodes in the Graph
    graph: &'g DirectedGraph<N>,
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
        /// The Head of the Branch, i.e. the last shared Node before the Graph split
        head: N::Id,
        /// The two Sides of the Branch
        sides: (DirectedChain<'n, N>, DirectedChain<'n, N>),
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

impl<'g, N> DirectedChain<'g, N>
where
    N: GraphNode,
{
    /// Overwrites the End of the Chain with the given Id, the End is exclusive so it wont be
    /// returned when iterating over the Chain
    #[must_use]
    pub fn set_end(mut self, end: N::Id) -> Self {
        self.end = Some(end);
        self
    }

    /// Gets the next Entry in the current Chain of Nodes, which may be an actual Node or a Split in the Control-Flow
    pub fn next_entry(&mut self) -> Option<ChainEntry<'g, N>> {
        let next_id = self.next.take()?;

        if let Some(prev) = self.previous.take() {
            let prev_node = self.graph.get_node(&prev)?;
            match succ_type(prev_node, self.graph) {
                None => {}
                Some(SuccType::Single(_)) => {}
                Some(SuccType::Branched { sides, end }) => {
                    self.next = Some(next_id);
                    return Some(ChainEntry::Branched {
                        head: prev,
                        sides: (
                            self.graph.chain_from(sides.0).set_end(end),
                            self.graph.chain_from(sides.1).set_end(end),
                        ),
                    });
                }
                Some(SuccType::Cycle { inner, .. }) => {
                    self.next = Some(next_id);

                    return Some(ChainEntry::Cycle {
                        head: prev,
                        inner: self.graph.chain_from(inner).set_end(prev),
                    });
                }
            };
        }

        self.previous = Some(next_id);

        let next_node = self.graph.get_node(&next_id)?;
        match succ_type(next_node, self.graph) {
            None => Some(ChainEntry::Node(next_node)),
            Some(SuccType::Single(succ_id)) => {
                if Some(succ_id) != self.end {
                    self.next = Some(succ_id);
                }

                Some(ChainEntry::Node(next_node))
            }
            Some(SuccType::Branched { end, .. }) => {
                self.next = Some(end);

                Some(ChainEntry::Node(next_node))
            }
            Some(SuccType::Cycle { following, .. }) => {
                self.next = Some(following);

                Some(ChainEntry::Node(next_node))
            }
        }
    }

    /// This creates a new identical Chain that returns the same Entries as the original Chain
    /// will return without modifying any of the original Chain's state
    pub fn duplicate(&self) -> Self {
        Self {
            previous: self.previous,
            next: self.next,
            end: self.end,
            graph: self.graph,
        }
    }

    /// The Graph to which this chain belongs
    pub fn graph(&self) -> &'g DirectedGraph<N> {
        self.graph
    }

    /// Turns the current Chain into a flattened Chain
    pub fn flatten(self) -> DirectedFlatChain<'g, N> {
        DirectedFlatChain::new(self)
    }
}

impl<'g, N> Iterator for DirectedChain<'g, N>
where
    N: GraphNode,
{
    type Item = ChainEntry<'g, N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_entry()
    }
}

/// A Chain that Flattens all control Structures in its root chain and therefore only returns
/// entries
pub struct DirectedFlatChain<'g, N>
where
    N: GraphNode,
{
    root_chain: DirectedChain<'g, N>,
    graph: &'g DirectedGraph<N>,
    pending: Vec<N::Id>,
}

impl<'g, N> DirectedFlatChain<'g, N>
where
    N: GraphNode,
{
    /// Creates a new Flat-Chain based on the given Chain
    pub fn new(chain: DirectedChain<'g, N>) -> Self {
        Self {
            graph: chain.graph,
            root_chain: chain,
            pending: Vec::new(),
        }
    }

    /// Obtains the next Entry in the Chain
    pub fn next_entry(&mut self) -> Option<&'g N> {
        if let Some(pended) = self.pending.pop() {
            return self.graph.get_node(&pended);
        }

        let raw_next = self.root_chain.next_entry()?;

        match raw_next {
            ChainEntry::Node(n) => Some(n),
            ChainEntry::Branched {
                sides: (left, right),
                ..
            } => {
                let mut left_flat = DirectedFlatChain::new(left);
                while let Some(b) = left_flat.next_entry() {
                    self.pending.push(b.id());
                }

                let mut right_flat = DirectedFlatChain::new(right);
                while let Some(b) = right_flat.next_entry() {
                    self.pending.push(b.id());
                }

                self.pending
                    .pop()
                    .map(|id| self.graph.get_node(&id).unwrap())
            }
            ChainEntry::Cycle { inner, .. } => {
                let mut inner_flat = DirectedFlatChain::new(inner);
                while let Some(b) = inner_flat.next_entry() {
                    self.pending.push(b.id());
                }

                self.pending
                    .pop()
                    .map(|id| self.graph.get_node(&id).unwrap())
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

#[derive(Debug, PartialEq)]
enum SuccType<I> {
    Single(I),
    Branched { sides: (I, I), end: I },
    Cycle { inner: I, following: I },
}

/// Used to determine the Type of Successors
/// * Single
/// * Branched
/// * Cycle
fn succ_type<N>(start_node: &N, graph: &DirectedGraph<N>) -> Option<SuccType<N::Id>>
where
    N: GraphNode,
{
    let start = start_node.id();
    let mut successors = start_node.successors();

    let mut visited: HashSet<N::Id> = HashSet::new();
    visited.insert(start);

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
            remaining.extend(tmp.successors().filter(|i| !first_ids.contains(i)));
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

mod mocks {
    use super::GraphNode;

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct MockNode {
        pub id: usize,
        pub successors: Vec<usize>,
    }

    impl GraphNode for MockNode {
        type Id = usize;
        type SuccessorIterator = std::vec::IntoIter<usize>;

        fn id(&self) -> Self::Id {
            self.id
        }

        fn successors(&self) -> Self::SuccessorIterator {
            self.successors.clone().into_iter()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let result = succ_type(graph.get_node(&0).unwrap(), &graph);
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

        let result = succ_type(graph.get_node(&0).unwrap(), &graph);
        assert_eq!(
            Some(SuccType::Branched {
                sides: (1, 2),
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
            let (head, mut left, mut right) = match entry {
                ChainEntry::Branched { head, sides } => (head, sides.0, sides.1),
                _ => {
                    panic!("Expected a Branched Entry");
                }
            };

            assert_eq!(0, head);

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

        let result = succ_type(graph.get_node(&0).unwrap(), &graph);
        assert_eq!(
            Some(SuccType::Cycle {
                inner: 1,
                following: 2
            }),
            result
        );

        let mut chain = graph.chain_iter();

        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_none());
        }

        // TODO
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

        let result = succ_type(graph.get_node(&0).unwrap(), &graph);
        assert_eq!(
            Some(SuccType::Cycle {
                inner: 2,
                following: 1
            }),
            result
        );

        let mut chain = graph.chain_iter();

        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_some());
        }
        {
            let raw_entry = chain.next_entry();
            assert!(raw_entry.is_none());
        }

        // TODO
    }
}
