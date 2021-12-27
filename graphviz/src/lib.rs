#![warn(missing_docs)]
//! This Crate provides an easy and simple way to create Graph visualizations using the Graphviz
//! dot format

mod node;
pub use node::*;

mod edge;
pub use edge::*;

mod subgraph;
pub use subgraph::*;

mod graph;
pub use graph::*;

mod args;

#[derive(Debug)]
enum Entry {
    Node(Node),
    Edge(Edge),
    SubGraph(SubGraph),
}

impl Entry {
    pub fn line(&self) -> String {
        match self {
            Self::Node(node) => node.line(),
            Self::Edge(edge) => edge.line(),
            Self::SubGraph(graph) => graph.line(),
        }
    }
}

/// This Trait is used to abstract over a Graph, which makes it easier to work with nested Graphs
/// that may contain Subgraphs, as they also implement this trait and can then used basically the
/// same as the Root Graph
pub trait Graph {
    /// Adds the given Node to the Graph
    fn add_node(&mut self, node: Node);

    /// Adds the given Edge to the Graph
    fn add_edge(&mut self, edge: Edge);

    /// Adds the given Subgraph to the Graph itself
    fn add_subgraph(&mut self, graph: SubGraph);
}
