use std::fmt::Debug;

use super::{
    successor::{self, succ_type, SuccType},
    ChainEntry, DirectedFlatChain, DirectedGraph, GraphNode,
};

/// A Chain that can be used to Iterate over a Chain of Entries in Graph
pub struct DirectedChain<'g, N>
where
    N: GraphNode,
{
    /// The Result of the Previously determined successor Type
    n_previous: Option<PreviousSucc<N::Id>>,
    /// The ID of the next Entry we will return
    next: Option<N::Id>,
    /// The ID up until which  we should return Entries (exclusive)
    end: Option<N::Id>,
    /// The ID of the Head if we are in a Cycle
    head: Option<N::Id>,
    /// The HashMap of Nodes in the Graph
    graph: &'g DirectedGraph<N>,
}

/// The Result of the previous Call to the [`succ_type`] function, to allow for easy caching and
/// later of use the returned Result
#[derive(Debug, Clone, Copy, PartialEq)]
enum PreviousSucc<I> {
    /// The next entry will be a Node
    Node,
    /// After the current Entry we hit a Branch
    Branched { sides: (I, Option<I>), end: I },
    /// After the current Entry we hit a Cycle
    Cycle { head: I, inner: I },
}

impl<'g, N> DirectedChain<'g, N>
where
    N: GraphNode,
{
    pub(crate) fn new(graph: &'g DirectedGraph<N>, start: Option<N::Id>) -> Self {
        Self {
            next: start,
            n_previous: None,
            end: None,
            head: None,
            graph,
        }
    }

    /// Overwrites the End of the Chain with the given Id, the End is exclusive so it wont be
    /// returned when iterating over the Chain
    #[must_use]
    pub fn set_end(mut self, end: N::Id) -> Self {
        self.end = Some(end);
        self
    }

    /// Gets the next Entry in the current Chain of Nodes, which may be an actual Node or a Split in the Control-Flow
    pub fn next_entry(&mut self) -> Option<ChainEntry<'g, N>> {
        let next_id = self.next.take();

        let context = if let Some(end) = self.end {
            successor::Context::OuterGraph { head: end }
        } else {
            successor::Context::None
        };

        if let Some(prev) = self.n_previous.take() {
            match prev {
                PreviousSucc::Node => {}
                PreviousSucc::Branched { sides, end } => {
                    self.next = next_id;

                    return Some(ChainEntry::Branched {
                        sides: (
                            self.graph.chain_from(sides.0).set_end(end),
                            sides.1.map(|s| self.graph.chain_from(s).set_end(end)),
                        ),
                    });
                }
                PreviousSucc::Cycle { head, inner } => {
                    self.next = next_id;

                    let inner = self.graph.chain_from(inner).set_end(head);

                    return Some(ChainEntry::Cycle { head, inner });
                }
            };
        }

        let next_id = next_id?;

        let next_node = self.graph.get_node(&next_id)?;
        match succ_type(next_node, self.graph, context) {
            None => Some(ChainEntry::Node(next_node)),
            Some(SuccType::Single(succ_id)) => {
                if Some(succ_id) != self.end {
                    self.next = Some(succ_id);
                }

                self.n_previous = Some(PreviousSucc::Node);

                Some(ChainEntry::Node(next_node))
            }
            Some(SuccType::Branched { end, sides }) => {
                self.next = Some(end);

                self.n_previous = Some(PreviousSucc::Branched { sides, end });

                Some(ChainEntry::Node(next_node))
            }
            Some(SuccType::Cycle { following, inner }) => {
                self.next = following;

                self.n_previous = Some(PreviousSucc::Cycle {
                    head: next_id,
                    inner,
                });

                Some(ChainEntry::Node(next_node))
            }
        }
    }

    /// This creates a new identical Chain that returns the same Entries as the original Chain
    /// will return without modifying any of the original Chain's state
    pub fn duplicate(&self) -> Self {
        Self {
            n_previous: self.n_previous,
            next: self.next,
            end: self.end,
            head: self.head,
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

impl<'g, N> Debug for DirectedChain<'g, N>
where
    N: GraphNode,
    N::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectedChain").finish()
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
