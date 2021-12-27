use std::collections::BTreeMap;

use super::{Entry, Graph};

/// A Subgraph used for building up nested Graphs
#[derive(Debug)]
pub struct SubGraph {
    entries: Vec<Entry>,
    name: String,
    cluster: bool,
    args: BTreeMap<String, String>,
}

impl SubGraph {
    /// Creates a new empty Subgraph
    pub fn new<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            entries: Vec::new(),
            name: name.into(),
            cluster: false,
            args: BTreeMap::new(),
        }
    }

    /// Marks this Subgraph as a Cluster, which will affect the final layout in the Dot format to
    /// force all the Nodes and Edges in this Subgraph to be grouped together
    pub fn cluster(mut self) -> Self {
        self.cluster = true;
        self
    }

    /// Adds some Args to the top level Graph to set some "Global" configurations for the entire
    /// Graph
    pub fn arg<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        self.args.insert(name.into(), value.into());
        self
    }

    /// Converts the Subgraph into its Text based representation
    pub fn line(&self) -> String {
        let mut lines = String::new();
        for entry in self.entries.iter() {
            let line = entry.line();
            lines.push_str(&line);
        }
        for (arg_name, arg_value) in self.args.iter() {
            let line = format!("{} = \"{}\";\n", arg_name, arg_value);
            lines.push_str(&line);
        }

        let graph_name = if self.cluster {
            format!("cluster_{}", self.name)
        } else {
            self.name.clone()
        };

        format!("subgraph {} {{\n{}}}\n", graph_name, lines)
    }
}

impl Graph for SubGraph {
    fn add_node(&mut self, node: super::Node) {
        self.entries.push(Entry::Node(node));
    }

    fn add_edge(&mut self, edge: super::Edge) {
        self.entries.push(Entry::Edge(edge));
    }

    fn add_subgraph(&mut self, graph: SubGraph) {
        self.entries.push(Entry::SubGraph(graph));
    }
}
