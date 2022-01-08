use crate::{DominanceTree, Variable};

pub struct PostOrderDominance<'t> {
    tree: &'t DominanceTree,
    next_node: Option<usize>,
}

impl<'t> PostOrderDominance<'t> {
    pub fn new(base: &'t DominanceTree) -> Self {
        let start_node = match base.root.clone() {
            Some(root_index) => {
                let final_index = Self::find_first_child(base, root_index);

                Some(final_index)
            }
            None => None,
        };

        Self {
            tree: base,
            next_node: start_node,
        }
    }

    fn find_first_child(tree: &DominanceTree, start: usize) -> usize {
        let mut final_index = start;
        let mut node = tree.nodes.get(final_index).unwrap();

        while !node.children.is_empty() {
            let next_index = *node.children.get(0).unwrap();
            final_index = next_index;
            node = tree.nodes.get(final_index).unwrap();
        }

        final_index
    }
}

impl Iterator for PostOrderDominance<'_> {
    type Item = Variable;

    fn next(&mut self) -> Option<Self::Item> {
        let current_index = self.next_node?;
        let current_node = self.tree.nodes.get(current_index).unwrap();

        let next_index = match current_node.parent {
            Some(p_index) => {
                let parent_node = self.tree.nodes.get(p_index).unwrap();
                let child_index = parent_node
                    .children
                    .iter()
                    .enumerate()
                    .find(|(_, c)| **c == current_index)
                    .map(|(i, _)| i)
                    .unwrap();
                let next_child_index = child_index + 1;

                if next_child_index < parent_node.children.len() {
                    let base_next_node_index = parent_node.children.get(next_child_index).unwrap();

                    let next_node_index = Self::find_first_child(self.tree, *base_next_node_index);
                    Some(next_node_index)
                } else {
                    Some(p_index)
                }
            }
            None => None,
        };
        self.next_node = next_index;

        Some(current_node.var.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::Type;

    use super::*;

    #[test]
    fn empty_iter() {
        let tree = DominanceTree::new();

        let mut iter = tree.post_order_iter();

        assert_eq!(None, iter.next());
    }

    #[test]
    fn iter_append_only() {
        let var_1 = Variable::new("1", Type::I8);
        let var_2 = Variable::new("2", Type::I8);
        let var_3 = Variable::new("3", Type::I8);
        let var_4 = Variable::new("4", Type::I8);

        let mut tree = DominanceTree::new();
        tree.append(var_1.clone());
        tree.append(var_2.clone());
        tree.append(var_3.clone());
        tree.append(var_4.clone());

        let mut iter = tree.post_order_iter();

        assert_eq!(Some(var_4), iter.next());
        assert_eq!(Some(var_3), iter.next());
        assert_eq!(Some(var_2), iter.next());
        assert_eq!(Some(var_1), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn iter_mixed() {
        let var_1 = Variable::new("1", Type::I8);
        let var_2 = Variable::new("2", Type::I8);
        let var_3 = Variable::new("3", Type::I8);
        let var_4 = Variable::new("4", Type::I8);

        let mut tree = DominanceTree::new();
        tree.append(var_1.clone());
        tree.append(var_2.clone());
        tree.append(var_3.clone());
        tree.insert_at_level(var_4.clone());

        let mut iter = tree.post_order_iter();

        assert_eq!(Some(var_3), iter.next());
        assert_eq!(Some(var_4), iter.next());
        assert_eq!(Some(var_2), iter.next());
        assert_eq!(Some(var_1), iter.next());
        assert_eq!(None, iter.next());
    }
}
