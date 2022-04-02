use crate::Variable;

mod postorder;
pub use postorder::*;

#[derive(Debug, PartialEq, Clone)]
struct DominanceNode {
    var: Variable,
    parent: Option<usize>,
    children: Vec<usize>,
}

impl DominanceNode {
    fn new(var: Variable, parent: Option<usize>) -> Self {
        Self {
            var,
            parent,
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, node: usize) {
        self.children.push(node);
    }
}

/// This struct holds the Information to identify the current Node, which can be used to change the
/// Level at which new Nodes are inserted at and allow for more than just one "Branch" in the Tree
pub struct CurrentNode {
    index: Option<usize>,
}

/// A simple Dominance Tree that represents which Variables dominate which other Variables
#[derive(Debug, PartialEq, Clone)]
pub struct DominanceTree {
    roots: Vec<usize>,
    nodes: Vec<DominanceNode>,
    latest: Option<usize>,
}

impl DominanceTree {
    /// Creates a new empty Dominance Tree
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            nodes: Vec::new(),
            latest: None,
        }
    }

    /// Returns an Iterator that traverses the Tree in Post Order
    pub fn post_order_iter(&self) -> PostOrderDominance<'_> {
        PostOrderDominance::new(self)
    }

    /// Inserts the new Variable at the same level as the last one and moves the current insertion
    /// Point to the new Node
    pub fn insert_at_level(&mut self, var: Variable) {
        let parent = match &self.latest {
            Some(latest) => self.nodes.get(*latest).unwrap().parent,
            None => panic!("Cant have two roots"),
        };
        let node = DominanceNode::new(var, parent);
        self.nodes.push(node);
        let node_index = self.nodes.len() - 1;

        if let Some(parent_index) = parent {
            let parent_node = self.nodes.get_mut(parent_index).unwrap();
            parent_node.add_child(node_index);
        }

        self.latest = Some(node_index);
    }

    /// Appends a new Node to the current One, which creates a new "Level" in the Tree.
    /// This also moves the current insertion Point to the newly added Node
    pub fn append(&mut self, var: Variable) {
        let parent = self.latest;
        let node = DominanceNode::new(var, parent);
        self.nodes.push(node);
        let node_index = self.nodes.len() - 1;

        if self.roots.is_empty() {
            self.roots.push(node_index);
        }
        if let Some(parent_index) = parent {
            let parent_node = self.nodes.get_mut(parent_index).unwrap();
            parent_node.add_child(node_index);
        }

        self.latest = Some(node_index);
    }

    /// Appends the given Tree to the current Node, where the Root of the other Tree is added like
    /// it was added using the [`append`] Method. The Rest of the other Tree is then kept in the
    /// same order starting from the inserted Root Node
    pub fn append_tree(&mut self, other: Self) {
        if self.roots.is_empty() {
            *self = other;
            return;
        }
        if other.roots.is_empty() {
            return;
        }

        let offset = self.nodes.len();

        self.nodes.extend(other.nodes.into_iter().map(|mut n| {
            if let Some(parent) = &mut n.parent {
                *parent += offset;
            }

            for child in n.children.iter_mut() {
                *child += offset;
            }

            n
        }));

        for other_root_index in other.roots.iter() {
            let other_root = self.nodes.get_mut(other_root_index + offset).unwrap();
            other_root.parent = self.latest;
        }

        for other_root_index in other.roots.iter() {
            match self.latest {
                Some(latest_index) => {
                    let latest = self.nodes.get_mut(latest_index).unwrap();
                    latest.add_child(other_root_index + offset);
                }
                None => {
                    self.roots.push(other_root_index + offset);
                }
            };
        }

        self.latest = other.latest.map(|prev| prev + offset);
    }

    /// Adds the other Tree starting at the same Level as the current insertion Point. The Root of
    /// the other Tree is added like when using the [`ìnsert_at_level`] function and then starting
    /// from that Point, the other Tree added with the same order as it was in previously
    pub fn insert_tree_at_level(&mut self, other: Self) {
        let parent = match &self.latest {
            Some(latest) => self.nodes.get(*latest).unwrap().parent,
            None => panic!("Cant have two roots"),
        };
        if other.roots.is_empty() {
            return;
        }

        let offset = self.nodes.len();

        self.nodes.extend(other.nodes.into_iter().map(|mut n| {
            if let Some(parent) = &mut n.parent {
                *parent += offset;
            }

            for child in n.children.iter_mut() {
                *child += offset;
            }

            n
        }));

        for other_root_index in other.roots.iter() {
            let other_root = self.nodes.get_mut(other_root_index + offset).unwrap();
            other_root.parent = parent;
        }

        match parent {
            Some(p_index) => {
                let n_p = self.nodes.get_mut(p_index).unwrap();

                for other_root in other.roots.iter() {
                    n_p.add_child(other_root + offset);
                }
            }
            None => panic!(),
        };

        self.latest = other.latest.map(|prev| prev + offset);
    }

    /// Returns a Handle to identify the current Node
    pub fn current_node(&self) -> CurrentNode {
        let latest = match self.latest {
            Some(l) => l,
            None => return CurrentNode { index: None },
        };

        CurrentNode {
            index: Some(latest),
        }
    }

    /// This will append the other Tree to the given Node, like it was described in [`append_tree`]
    /// but starting from the given Node
    pub fn append_tree_to_node(&mut self, node: &CurrentNode, other: Self) {
        self.latest = node.index;

        self.append_tree(other);
    }

    /// Moves the current insertion Point to the given Node
    pub fn move_to_node(&mut self, node: CurrentNode) {
        self.latest = node.index;
    }

    /// Searches for the given Variable in the Parents of the current Node
    pub fn search_parents(&mut self, var: &Variable) -> Option<CurrentNode> {
        let start = self.latest?;

        let mut current_i = start;
        loop {
            let current = self.nodes.get(current_i).unwrap();
            if &current.var == var {
                return Some(CurrentNode {
                    index: Some(current_i),
                });
            }

            match current.parent {
                Some(p) => {
                    current_i = p;
                }
                None => return None,
            };
        }
    }
}

impl Default for DominanceTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::Type;

    use super::*;

    #[test]
    fn new() {
        let tree = DominanceTree::new();
        dbg!(&tree);
    }

    #[test]
    fn append() {
        let first_var = Variable::new("test", Type::I8);
        let second_var = Variable::new("test2", Type::I8);

        let mut tree = DominanceTree::new();
        tree.append(first_var.clone());
        tree.append(second_var.clone());

        let mut root_node = DominanceNode::new(first_var, None);
        root_node.add_child(1);

        let expected = DominanceTree {
            roots: vec![0],
            latest: Some(1),
            nodes: vec![root_node, DominanceNode::new(second_var, Some(0))],
        };

        assert_eq!(expected, tree);
    }

    #[test]
    fn append_tree() {
        let x = Variable::new("x", Type::I8);
        let y = Variable::new("y", Type::I8);
        let z = Variable::new("z", Type::I8);
        let root = Variable::new("root", Type::I8);

        let mut other_tree = DominanceTree::new();
        other_tree.append(x.clone());
        other_tree.append(y.clone());
        other_tree.insert_at_level(z.clone());

        let mut result_tree = DominanceTree::new();
        result_tree.append(root.clone());
        result_tree.append_tree(other_tree);
        dbg!(&result_tree);

        let mut expected_root = DominanceNode::new(root, None);
        expected_root.add_child(1);
        let mut expected_x = DominanceNode::new(x, Some(0));
        expected_x.add_child(2);
        expected_x.add_child(3);
        let expected_y = DominanceNode::new(y, Some(1));
        let expected_z = DominanceNode::new(z, Some(1));

        let expected_tree = DominanceTree {
            roots: vec![0],
            latest: Some(3),
            nodes: vec![expected_root, expected_x, expected_y, expected_z],
        };

        assert_eq!(expected_tree, result_tree);
    }

    #[test]
    fn append_tree_to_node() {
        let x = Variable::new("x", Type::I8);
        let y = Variable::new("y", Type::I8);
        let z = Variable::new("z", Type::I8);
        let root = Variable::new("root", Type::I8);
        let root2 = Variable::new("root2", Type::I8);

        let mut other_tree = DominanceTree::new();
        other_tree.append(x.clone());
        other_tree.append(y.clone());
        other_tree.insert_at_level(z.clone());

        let mut result_tree = DominanceTree::new();
        result_tree.append(root.clone());
        let result_root_node = result_tree.current_node();
        result_tree.append(root2.clone());
        result_tree.append_tree_to_node(&result_root_node, other_tree);
        dbg!(&result_tree);

        let mut expected_root = DominanceNode::new(root, None);
        expected_root.add_child(1);
        expected_root.add_child(2);
        let expected_root_2 = DominanceNode::new(root2, Some(0));
        let mut expected_x = DominanceNode::new(x, Some(0));
        expected_x.add_child(3);
        expected_x.add_child(4);
        let expected_y = DominanceNode::new(y, Some(2));
        let expected_z = DominanceNode::new(z, Some(2));

        let expected_tree = DominanceTree {
            roots: vec![0],
            latest: Some(4),
            nodes: vec![
                expected_root,
                expected_root_2,
                expected_x,
                expected_y,
                expected_z,
            ],
        };

        assert_eq!(expected_tree, result_tree);
    }

    #[test]
    fn insert_level() {
        let first_var = Variable::new("test", Type::I8);
        let second_var = Variable::new("test2", Type::I8);
        let third_var = Variable::new("test3", Type::I8);

        let mut tree = DominanceTree::new();
        tree.append(first_var.clone());
        tree.append(second_var.clone());
        tree.insert_at_level(third_var.clone());

        let mut root_node = DominanceNode::new(first_var, None);
        root_node.add_child(1);
        root_node.add_child(2);

        let expected = DominanceTree {
            roots: vec![0],
            latest: Some(2),
            nodes: vec![
                root_node,
                DominanceNode::new(second_var, Some(0)),
                DominanceNode::new(third_var, Some(0)),
            ],
        };

        assert_eq!(expected, tree);
    }

    #[test]
    fn insert_tree() {
        let x = Variable::new("x", Type::I8);
        let y = Variable::new("y", Type::I8);
        let z = Variable::new("z", Type::I8);
        let root = Variable::new("root", Type::I8);
        let root2 = Variable::new("root2", Type::I8);

        let mut other_tree = DominanceTree::new();
        other_tree.append(x.clone());
        other_tree.append(y.clone());
        other_tree.insert_at_level(z.clone());

        let mut result_tree = DominanceTree::new();
        result_tree.append(root.clone());
        result_tree.append(root2.clone());
        result_tree.insert_tree_at_level(other_tree);
        dbg!(&result_tree);

        let mut expected_root = DominanceNode::new(root, None);
        expected_root.add_child(1);
        expected_root.add_child(2);
        let expected_root_2 = DominanceNode::new(root2, Some(0));
        let mut expected_x = DominanceNode::new(x, Some(0));
        expected_x.add_child(3);
        expected_x.add_child(4);
        let expected_y = DominanceNode::new(y, Some(2));
        let expected_z = DominanceNode::new(z, Some(2));

        let expected_tree = DominanceTree {
            roots: vec![0],
            latest: Some(4),
            nodes: vec![
                expected_root,
                expected_root_2,
                expected_x,
                expected_y,
                expected_z,
            ],
        };

        assert_eq!(expected_tree, result_tree);
    }

    #[test]
    fn append_at_root() {
        let x = Variable::new("x", Type::I8);
        let y = Variable::new("x", Type::I8);

        let mut result_tree = DominanceTree::new();
        let root_node = result_tree.current_node();

        let other_tree = {
            let mut tmp = DominanceTree::new();
            tmp.append(x.clone());
            tmp.append(y.clone());
            tmp
        };
        result_tree.append_tree_to_node(&root_node, other_tree.clone());
        result_tree.append_tree_to_node(&root_node, other_tree);

        let expected = DominanceTree {
            roots: vec![0, 2],
            latest: Some(3),
            nodes: vec![
                DominanceNode {
                    var: x.clone(),
                    parent: None,
                    children: vec![1],
                },
                DominanceNode {
                    var: y.clone(),
                    parent: Some(0),
                    children: vec![],
                },
                DominanceNode {
                    var: x,
                    parent: None,
                    children: vec![3],
                },
                DominanceNode {
                    var: y,
                    parent: Some(2),
                    children: vec![],
                },
            ],
        };

        assert_eq!(expected, result_tree);
    }
}
