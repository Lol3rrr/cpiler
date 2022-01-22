use std::collections::HashMap;

use ir::{BasicBlock, InnerBlock, Statement, WeakBlockPtr};

use crate::OptimizationPass;

/// This Optimization attempts to merge IR-Blocks together, which makes it easier for other
/// Optimizations as well as hopefully resulting in better generated Code as possibly reduces
/// the number of Jump instructions to connect the different Blocks
///
/// This is similiar to function inlining, just at the block level and only per function by default
pub struct Merger {}

impl Merger {
    /// Creates a new Merger Instance
    pub fn new() -> Self {
        Self {}
    }

    fn merge(&self, block: &BasicBlock, mappings: &mut HashMap<*const InnerBlock, WeakBlockPtr>) {
        if block.successors().len() > 1 {
            return;
        }

        let mut block_statements = block.get_statements();

        let last = match block_statements.pop() {
            Some(l) => l,
            None => return,
        };

        if let Statement::Jump(target) = last {
            self.merge(&target, mappings);

            let mut target_preds = target.get_predecessors();
            if target_preds.len() != 1 {
                return;
            }

            let target_pred = target_preds.remove(0);
            if target_pred != block.weak_ptr() {
                panic!()
            }

            let target_statements = target.get_statements();

            let block_ptr = block.weak_ptr();
            let target_ptr = target.weak_ptr();
            target_statements.iter().for_each(|stmnt| {
                match stmnt {
                    Statement::Jump(tmp) | Statement::JumpTrue(_, tmp) => {
                        tmp.remove_predecessor(target_ptr.clone());
                        tmp.add_predecessor(block_ptr.clone());
                    }
                    _ => {}
                };
            });

            mappings.insert(target_ptr.as_ptr(), block_ptr);

            let merged: Vec<_> = block_statements
                .into_iter()
                .chain(target_statements.into_iter())
                .collect();

            block.set_statements(merged);
        }
    }

    fn phis(&self, block: &BasicBlock, mappings: &HashMap<*const InnerBlock, WeakBlockPtr>) {
        let mut statements = block.get_statements();

        for tmp in statements.iter_mut() {
            match tmp {
                Statement::Assignment {
                    target,
                    value: ir::Value::Phi { sources },
                } => {
                    let n_sources = sources
                        .clone()
                        .into_iter()
                        .map(|s| {
                            let mut last = s;
                            loop {
                                match mappings.get(&last.block.as_ptr()) {
                                    Some(n) => {
                                        last.block = n.clone();
                                    }
                                    None => break,
                                };
                            }
                            last
                        })
                        .collect();

                    *tmp = Statement::Assignment {
                        target: target.clone(),
                        value: ir::Value::Phi { sources: n_sources },
                    };
                }
                _ => {}
            };
        }

        block.set_statements(statements);
    }
}

impl Default for Merger {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for Merger {
    fn name(&self) -> String {
        "Merger".to_string()
    }

    fn pass_function(&self, ir: ir::FunctionDefinition) -> ir::FunctionDefinition {
        let mut merge_mappings = HashMap::new();

        self.merge(&ir.block, &mut merge_mappings);

        for tmp in ir.block.block_iter() {
            self.phis(&tmp, &merge_mappings);
        }

        ir
    }
}

#[cfg(test)]
mod tests {
    use ir::{BasicBlock, Statement};

    use super::*;

    #[test]
    fn two_valid_block() {
        let initial = BasicBlock::initial(vec![]);

        let second_block = BasicBlock::new(vec![initial.weak_ptr()], vec![Statement::Return(None)]);
        initial.add_statement(Statement::Jump(second_block));

        let previous = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: ir::Type::Void,
            block: initial,
        };

        let expected_block = BasicBlock::initial(vec![Statement::Return(None)]);
        let expected = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: ir::Type::Void,
            block: expected_block,
        };

        let pass = Merger::new();

        let result = pass.pass_function(previous);
        dbg!(&result);

        assert_eq!(expected, result);
    }

    #[test]
    fn three_valid_blocks() {
        let initial = BasicBlock::initial(vec![]);

        let second_block = BasicBlock::new(vec![initial.weak_ptr()], vec![]);
        initial.add_statement(Statement::Jump(second_block.clone()));

        let third_block =
            BasicBlock::new(vec![second_block.weak_ptr()], vec![Statement::Return(None)]);
        second_block.add_statement(Statement::Jump(third_block));

        let previous = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: ir::Type::Void,
            block: initial,
        };

        let expected_block = BasicBlock::initial(vec![Statement::Return(None)]);
        let expected = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: ir::Type::Void,
            block: expected_block,
        };

        let pass = Merger::new();

        let result = pass.pass_function(previous);
        dbg!(&result);

        assert_eq!(expected, result);
    }

    #[test]
    fn merge_middle_block() {
        let initial = BasicBlock::initial(vec![]);

        let second_block = BasicBlock::new(vec![initial.weak_ptr()], vec![]);

        let third_block = BasicBlock::new(
            vec![initial.weak_ptr(), second_block.weak_ptr()],
            vec![Statement::Return(None)],
        );
        initial.add_statement(Statement::Jump(third_block.clone()));
        initial.add_statement(Statement::Jump(second_block.clone()));
        second_block.add_statement(Statement::Jump(third_block));

        let previous = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: ir::Type::Void,
            block: initial,
        };

        let expected_initial = BasicBlock::initial(vec![]);
        let expected_last = BasicBlock::new(
            vec![expected_initial.weak_ptr()],
            vec![Statement::Return(None)],
        );
        expected_initial.add_statement(Statement::Jump(expected_last.clone()));
        expected_initial.add_statement(Statement::Jump(expected_last));

        let expected = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: ir::Type::Void,
            block: expected_initial,
        };

        let pass = Merger::new();

        dbg!(&previous);
        let result = pass.pass_function(previous);
        dbg!(&result);

        assert_eq!(expected, result);
    }
}
