use graphs::directed::{ChainEntry, DirectedChain, DirectedGraph};

use crate::{BasicBlock, DominanceTree, Statement};

fn generate_chain(mut chain: DirectedChain<'_, BasicBlock>) -> DominanceTree {
    let mut tree = DominanceTree::new();

    while let Some(entry) = chain.next_entry() {
        match entry {
            ChainEntry::Node(node) => {
                for stmnt in node.get_statements() {
                    if let Statement::Assignment { target, .. } = stmnt {
                        tree.append(target);
                    }
                }
            }
            ChainEntry::Branched {
                sides: (left, right),
                ..
            } => {
                let current = tree.current_node();

                let left_tree = generate_chain(left);

                tree.append_tree_to_node(&current, left_tree);
                if let Some(right) = right {
                    let right_tree = generate_chain(right);
                    tree.append_tree_to_node(&current, right_tree);
                }

                tree.move_to_node(current);
            }
            ChainEntry::Cycle { inner, .. } => {
                let current = tree.current_node();

                let inner_tree = generate_chain(inner);

                tree.append_tree_to_node(&current, inner_tree);

                tree.move_to_node(current);
            }
        };
    }

    tree
}

pub fn generate(graph: &DirectedGraph<BasicBlock>) -> DominanceTree {
    let chain = graph.chain_iter();
    generate_chain(chain)
}
