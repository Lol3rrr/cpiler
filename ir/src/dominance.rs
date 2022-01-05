use crate::Variable;

mod postorder;
pub use postorder::*;

#[derive(Debug, PartialEq)]
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

pub struct CurrentNode {
    index: usize,
}

#[derive(Debug, PartialEq)]
pub struct DominanceTree {
    root: Option<usize>,
    nodes: Vec<DominanceNode>,
    latest: Option<usize>,
}

impl DominanceTree {
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: Vec::new(),
            latest: None,
        }
    }

    pub fn post_order_iter(&self) -> PostOrderDominance<'_> {
        PostOrderDominance::new(self)
    }

    /// Inserts the new Variable at the same level as the last one
    pub fn insert_at_level(&mut self, var: Variable) {
        let parent = match &self.latest {
            Some(latest) => self.nodes.get(*latest).unwrap().parent.clone(),
            None => panic!("Cant have two roots"),
        };

        let node = DominanceNode::new(var, parent.clone());
        self.nodes.push(node);
        let node_index = self.nodes.len() - 1;

        if let Some(parent_index) = parent {
            let parent_node = self.nodes.get_mut(parent_index).unwrap();
            parent_node.add_child(node_index);
        }

        self.latest = Some(node_index);
    }

    pub fn append(&mut self, var: Variable) {
        let parent = self.latest.clone();
        let node = DominanceNode::new(var, parent);
        self.nodes.push(node);
        let node_index = self.nodes.len() - 1;

        if self.root.is_none() {
            self.root = Some(node_index);
        }
        if let Some(parent_index) = parent {
            let parent_node = self.nodes.get_mut(parent_index).unwrap();
            parent_node.add_child(node_index);
        }

        self.latest = Some(node_index);
    }

    pub fn append_tree(&mut self, other: Self) {
        if self.root.is_none() {
            *self = other;
            return;
        }
        if other.root.is_none() {
            return;
        }

        let offset = self.nodes.len();

        self.nodes.extend(other.nodes.into_iter().map(|mut n| {
            if let Some(parent) = &mut n.parent {
                *parent = *parent + offset;
            }

            for child in n.children.iter_mut() {
                *child = *child + offset;
            }

            n
        }));

        let other_root_index = other.root.unwrap();
        let other_root = self.nodes.get_mut(other_root_index + offset).unwrap();
        other_root.parent = self.latest.clone();

        let latest_index = self.latest.unwrap();
        let latest = self.nodes.get_mut(latest_index).unwrap();
        latest.add_child(other.root.unwrap() + offset);

        self.latest = other.latest.map(|prev| prev + offset);
    }

    pub fn insert_tree_at_level(&mut self, other: Self) {
        let parent = match &self.latest {
            Some(latest) => self.nodes.get(*latest).unwrap().parent.clone(),
            None => panic!("Cant have two roots"),
        };
        if other.root.is_none() {
            return;
        }

        let offset = self.nodes.len();

        self.nodes.extend(other.nodes.into_iter().map(|mut n| {
            if let Some(parent) = &mut n.parent {
                *parent = *parent + offset;
            }

            for child in n.children.iter_mut() {
                *child = *child + offset;
            }

            n
        }));

        let other_root_index = other.root.unwrap();
        let other_root = self.nodes.get_mut(other_root_index + offset).unwrap();
        other_root.parent = parent;

        match parent {
            Some(p_index) => {
                let n_p = self.nodes.get_mut(p_index).unwrap();
                n_p.add_child(other.root.unwrap() + offset);
            }
            None => panic!(),
        };

        self.latest = other.latest.map(|prev| prev + offset);
    }

    pub fn current_node(&self) -> Option<CurrentNode> {
        let latest = self.latest.clone()?;

        Some(CurrentNode { index: latest })
    }

    pub fn append_tree_to_node(&mut self, node: &CurrentNode, other: Self) {
        self.latest = Some(node.index);

        self.append_tree(other);
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
            root: Some(0),
            latest: Some(1),
            nodes: vec![root_node, DominanceNode::new(second_var.clone(), Some(0))],
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

        let mut expected_root = DominanceNode::new(root.clone(), None);
        expected_root.add_child(1);
        let mut expected_x = DominanceNode::new(x.clone(), Some(0));
        expected_x.add_child(2);
        expected_x.add_child(3);
        let expected_y = DominanceNode::new(y.clone(), Some(1));
        let expected_z = DominanceNode::new(z.clone(), Some(1));

        let expected_tree = DominanceTree {
            root: Some(0),
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
        let result_root_node = result_tree.current_node().unwrap();
        result_tree.append(root2.clone());
        result_tree.append_tree_to_node(&result_root_node, other_tree);
        dbg!(&result_tree);

        let mut expected_root = DominanceNode::new(root.clone(), None);
        expected_root.add_child(1);
        expected_root.add_child(2);
        let expected_root_2 = DominanceNode::new(root2.clone(), Some(0));
        let mut expected_x = DominanceNode::new(x.clone(), Some(0));
        expected_x.add_child(3);
        expected_x.add_child(4);
        let expected_y = DominanceNode::new(y.clone(), Some(2));
        let expected_z = DominanceNode::new(z.clone(), Some(2));

        let expected_tree = DominanceTree {
            root: Some(0),
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
            root: Some(0),
            latest: Some(2),
            nodes: vec![
                root_node,
                DominanceNode::new(second_var.clone(), Some(0)),
                DominanceNode::new(third_var.clone(), Some(0)),
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

        let mut expected_root = DominanceNode::new(root.clone(), None);
        expected_root.add_child(1);
        expected_root.add_child(2);
        let expected_root_2 = DominanceNode::new(root2.clone(), Some(0));
        let mut expected_x = DominanceNode::new(x.clone(), Some(0));
        expected_x.add_child(3);
        expected_x.add_child(4);
        let expected_y = DominanceNode::new(y.clone(), Some(2));
        let expected_z = DominanceNode::new(z.clone(), Some(2));

        let expected_tree = DominanceTree {
            root: Some(0),
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
}
