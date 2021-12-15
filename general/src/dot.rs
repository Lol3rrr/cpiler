mod node;
pub use node::Node;

mod edge;
pub use edge::Edge;

mod subgraph;
pub use subgraph::SubGraph;

mod graph;
pub use graph::RootGraph;

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

pub trait Graph {
    fn add_node(&mut self, node: Node);
    fn add_edge(&mut self, edge: Edge);
    fn add_subgraph(&mut self, graph: SubGraph);
}
