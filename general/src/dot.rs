use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Node {
    name: String,
    args: BTreeMap<String, String>,
}

impl Node {
    pub fn new<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            args: BTreeMap::new(),
        }
    }

    pub fn add_label<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        self.args.insert(name.into(), value.into());
        self
    }
}

#[derive(Debug)]
pub struct Edge {
    from: String,
    to: String,
    args: BTreeMap<String, String>,
}

impl Edge {
    pub fn new<S, D>(src: S, dest: D) -> Self
    where
        S: Into<String>,
        D: Into<String>,
    {
        Self {
            from: src.into(),
            to: dest.into(),
            args: BTreeMap::new(),
        }
    }

    pub fn add_label<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        self.args.insert(name.into(), value.into());
        self
    }
}

#[derive(Debug)]
enum Entry {
    Node(Node),
    Edge(Edge),
}

#[derive(Debug)]
pub struct Graph {
    entries: Vec<Entry>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.entries.push(Entry::Node(node));
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.entries.push(Entry::Edge(edge));
    }

    pub fn finalize(self) -> String {
        let mut result = "digraph G {\n".to_string();
        for entry in self.entries {
            result.push_str("  ");
            match entry {
                Entry::Node(node) => {
                    result.push_str(&node.name);

                    if !node.args.is_empty() {
                        result.push('[');
                        for (arg_name, arg_value) in node.args {
                            result.push_str(&arg_name);
                            result.push_str("=\"");
                            result.push_str(&arg_value);
                            result.push_str("\" ");
                        }
                        result.push(']');
                    }
                }
                Entry::Edge(edge) => {
                    result.push_str(&edge.from);
                    result.push_str(" -> ");
                    result.push_str(&edge.to);

                    if !edge.args.is_empty() {
                        result.push('[');
                        for (arg_name, arg_value) in edge.args {
                            result.push_str(&arg_name);
                            result.push_str("=\"");
                            result.push_str(&arg_value);
                            result.push_str("\" ");
                        }
                        result.push(']');
                    }
                }
            };
            result.push_str(";\n");
        }
        result.push('}');

        result
    }
}
