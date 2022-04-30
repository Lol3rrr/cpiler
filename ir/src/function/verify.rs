use std::collections::HashSet;

use graphs::directed::DirectedGraph;

use crate::{text_rep, BasicBlock, Statement};

pub fn verify(graph: DirectedGraph<BasicBlock>) {
    let mut assigned_vars = HashSet::new();

    for block in graph.chain_iter().flatten() {
        for stmnt in block.get_statements() {
            match stmnt {
                Statement::Assignment { target, .. } => {
                    if assigned_vars.contains(&target) {
                        println!("{}", text_rep::block_text_rep(block, "".to_string()));
                        panic!("{:?}", target);
                    }

                    assert!(assigned_vars.insert(target));
                }
                _ => {}
            };
        }
    }
}
