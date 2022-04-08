use std::{collections::HashMap, hash::Hash};

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
            nodes: &self.nodes,
        }
    }
}

pub struct DirectedChain<'g, N>
where
    N: GraphNode,
{
    /// The ID of the last Entry we returned
    previous: Option<N::Id>,
    /// The ID of the next Entry we will return
    next: Option<N::Id>,
    /// The ID of the last Entry we should return (inklusive)
    end: Option<N::Id>,
    /// The HashMap of Nodes in the Graph
    nodes: &'g HashMap<N::Id, N>,
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
    Branched(Vec<DirectedChain<'n, N>>),
    /// The Graph has encountered a Cycle, contains the Chain to go over all the Entries in the
    /// Body of the Cycle
    Cycle(DirectedChain<'n, N>),
}

impl<'g, N> DirectedChain<'g, N>
where
    N: GraphNode,
{
    /// Gets the next Entry in the current Chain of Nodes, which may be an actual Node or a Split in the Control-Flow
    pub fn next_entry(&mut self) -> Option<ChainEntry<'_, N>> {
        let next_id = self.next?;

        todo!()
    }
}

/// Used to determine the Type of Successors
/// * Single
/// * Branched
/// * Cycle
fn succ_type<N>(successors: impl Iterator<Item = N::Id>)
where
    N: GraphNode,
{
    todo!()
}
