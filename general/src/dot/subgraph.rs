use std::collections::BTreeMap;

use super::{Entry, Graph};

#[derive(Debug)]
pub struct SubGraph {
    entries: Vec<Entry>,
    name: String,
    cluster: bool,
    args: BTreeMap<String, String>,
}

impl SubGraph {
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

    pub fn cluster(mut self) -> Self {
        self.cluster = true;
        self
    }

    pub fn arg<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        self.args.insert(name.into(), value.into());
        self
    }

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
