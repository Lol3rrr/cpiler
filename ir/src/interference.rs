use std::collections::{HashMap, HashSet};

use graphviz::Graph;

use crate::Variable;

/// An ID for a single Node
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct NodeId {
    var: Variable,
}

impl NodeId {
    /// Generates the NodeID for the given Variable
    pub fn new(var: Variable) -> Self {
        Self { var }
    }

    /// Get the underlying Variable of this NodeID
    pub fn var(&self) -> &Variable {
        &self.var
    }
}

impl From<Variable> for NodeId {
    fn from(v: Variable) -> Self {
        Self::new(v)
    }
}
impl From<&Variable> for NodeId {
    fn from(v: &Variable) -> Self {
        Self::new(v.clone())
    }
}

/// This trait should be implemented by consumers of the IR that need to create an
/// interference Graph for a given IR.
///
/// This allows the actual generation of the IR to be independant of the underlying Graph
/// implementation which allows the Consumer to use a Datastructure best suited to their
/// given Use-Case
pub trait InterferenceGraph {
    /// Adds a new Node to the Interference Graph
    fn add_node<N>(&mut self, name: N)
    where
        N: Into<NodeId>;

    /// Adds a new Edge between two Nodes
    fn add_edge<F, S>(&mut self, first: F, second: S)
    where
        F: Into<NodeId>,
        S: Into<NodeId>;
}

/// A simple Datastructure for an Interference Graph, which can be used as is if you dont need any
/// special Properties from your Datastructure
#[derive(Debug)]
pub struct DefaultInterferenceGraph {
    /// The Nodes in the Graph
    pub nodes: HashSet<NodeId>,
    /// The Edges used in the Graph
    pub edges: Vec<(NodeId, NodeId)>,
}

impl DefaultInterferenceGraph {
    /// Creates a new Instance of the InterferenceGraph
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: Vec::new(),
        }
    }

    /// Convert the Graph into a Dot Graphviz format for easy visualization
    pub fn to_dot(&self) -> String {
        let mut graph = graphviz::RootGraph::new();

        let mut cons = HashMap::new();

        for node in self.nodes.iter() {
            let name = format!("{}_{}", node.var.name, node.var.generation());

            graph.add_node(graphviz::Node::new(name));

            cons.insert(node.clone(), HashSet::new());
        }

        for (first, second) in self.edges.iter() {
            let second_targets = cons.get(second).unwrap();
            if second_targets.contains(first) {
                continue;
            }

            let first_targets = cons.get_mut(first).unwrap();
            if first_targets.contains(second) {
                continue;
            }

            let first_name = format!("{}_{}", first.var.name, first.var.generation());
            let second_name = format!("{}_{}", second.var.name, second.var.generation());

            graph.add_edge(graphviz::Edge::new(&first_name, &second_name).add_label("dir", "none"));

            first_targets.insert(second.clone());
        }

        graph.finalize()
    }

    /// Get all the Nodes that are connected to the given Node
    pub fn neighbours<N>(&self, node: N) -> Vec<NodeId>
    where
        N: Into<NodeId>,
    {
        let node = node.into();
        let mut result = Vec::new();

        for (first, second) in self.edges.iter() {
            if first == &node {
                result.push(second.clone());
            }
            if second == &node {
                result.push(first.clone());
            }
        }

        result
    }
}

impl InterferenceGraph for DefaultInterferenceGraph {
    fn add_node<N>(&mut self, name: N)
    where
        N: Into<NodeId>,
    {
        self.nodes.insert(name.into());
    }

    fn add_edge<F, S>(&mut self, first: F, second: S)
    where
        F: Into<NodeId>,
        S: Into<NodeId>,
    {
        self.edges.push((first.into(), second.into()));
    }
}

impl PartialEq for DefaultInterferenceGraph {
    fn eq(&self, other: &Self) -> bool {
        if !self.nodes.eq(&other.nodes) {
            return false;
        }

        let gen_edges = |edges: &[(NodeId, NodeId)]| {
            let mut tmp: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();

            for (first, second) in edges.iter() {
                match tmp.get_mut(first) {
                    Some(first_targets) => {
                        first_targets.insert(second.clone());
                    }
                    None => {
                        let mut first_targets = HashSet::new();
                        first_targets.insert(second.clone());
                        tmp.insert(first.clone(), first_targets);
                    }
                };
                match tmp.get_mut(second) {
                    Some(second_targets) => {
                        second_targets.insert(first.clone());
                    }
                    None => {
                        let mut second_targets = HashSet::new();
                        second_targets.insert(first.clone());
                        tmp.insert(second.clone(), second_targets);
                    }
                };
            }

            tmp
        };

        let own_edges = gen_edges(&self.edges);
        let other_edges = gen_edges(&other.edges);

        dbg!(&other_edges);

        own_edges.eq(&other_edges)
    }
}

impl Default for DefaultInterferenceGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// An Interference Graph Datastructure that also keeps track of the largets Clique in the Graph,
/// which makes it a bit slower as the [`DefaultInterferenceGraph`] but this difference should
/// not be too dramatic in most cases
#[derive(Debug, PartialEq)]
pub struct MaxInterferenceGraph {
    nodes: HashSet<NodeId>,
    edges: HashMap<NodeId, HashSet<NodeId>>,
    largest: Option<(NodeId, usize)>,
}

impl MaxInterferenceGraph {
    /// Creates a new empty Graph Instance
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashMap::new(),
            largest: None,
        }
    }

    /// Returns an Iterator over the Nodes of the Graph
    pub fn nodes_iter(&self) -> impl Iterator<Item = &NodeId> + '_ {
        self.nodes.iter()
    }

    /// Returns an Iterator over all the Neighbouring Nodes of the given Node
    pub fn neighbours<N>(&self, node: N) -> Option<impl Iterator<Item = &NodeId> + '_>
    where
        N: Into<NodeId>,
    {
        let id = node.into();
        let entries = self.edges.get(&id)?;

        Some(entries.iter())
    }
}

impl InterferenceGraph for MaxInterferenceGraph {
    fn add_node<N>(&mut self, name: N)
    where
        N: Into<NodeId>,
    {
        let id = name.into();
        self.nodes.insert(id.clone());
        self.edges.insert(id, HashSet::new());
    }

    fn add_edge<F, S>(&mut self, first: F, second: S)
    where
        F: Into<NodeId>,
        S: Into<NodeId>,
    {
        let first_id = first.into();
        let second_id = second.into();

        let first_entry = self.edges.entry(first_id.clone());
        let first_edges = first_entry.or_insert_with(HashSet::new);
        first_edges.insert(second_id.clone());

        match self.largest.as_mut() {
            Some((var, count)) => {
                if first_edges.len() > *count {
                    *count = first_edges.len();
                    *var = first_id.clone();
                }
            }
            None => {
                self.largest = Some((first_id.clone(), first_edges.len()));
            }
        };

        let second_entry = self.edges.entry(second_id.clone());
        let second_edges = second_entry.or_insert_with(HashSet::new);
        second_edges.insert(first_id.clone());

        match self.largest.as_mut() {
            Some((var, count)) => {
                if second_edges.len() > *count {
                    *count = second_edges.len();
                    *var = second_id;
                }
            }
            None => {
                self.largest = Some((first_id, second_edges.len()));
            }
        };
    }
}

impl Default for MaxInterferenceGraph {
    fn default() -> Self {
        Self::new()
    }
}
