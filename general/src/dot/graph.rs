use super::{Edge, Entry, Graph, Node};

#[derive(Debug)]
pub struct RootGraph {
    entries: Vec<Entry>,
}

impl RootGraph {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn finalize(self) -> String {
        let mut result = "digraph G {\n".to_string();
        for entry in self.entries {
            let line = entry.line();
            result.push_str(&line);
        }
        result.push('}');

        result
    }
}

impl Graph for RootGraph {
    fn add_node(&mut self, node: Node) {
        self.entries.push(Entry::Node(node));
    }

    fn add_edge(&mut self, edge: Edge) {
        self.entries.push(Entry::Edge(edge));
    }

    fn add_subgraph(&mut self, graph: super::SubGraph) {
        self.entries.push(Entry::SubGraph(graph));
    }
}
