use std::collections::HashMap;

use ir::{Constant, Statement, Variable};

use crate::OptimizationPass;

#[derive(Debug)]
struct UseMap {
    inner: HashMap<ir::Variable, usize>,
}

impl UseMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn increment(&mut self, var: ir::Variable) {
        match self.inner.get_mut(&var) {
            Some(count) => {
                *count += 1;
            }
            None => {
                self.inner.insert(var, 1);
            }
        };
    }

    pub fn decrement(&mut self, var: &ir::Variable) {
        let n_count = match self.inner.get_mut(var) {
            Some(count) => {
                *count = (*count).saturating_sub(1);
                *count
            }
            None => 0,
        };

        if n_count == 0 {
            self.inner.remove(var);
        }
    }

    pub fn contains(&self, var: &ir::Variable) -> bool {
        self.inner.contains_key(var)
    }
}

/// Performs basic DeadCode Elimination
pub struct DeadCode {}

impl DeadCode {
    /// Creates a new DeadCode Pass Instance
    pub fn new() -> Self {
        Self {}
    }

    fn filter_stmnts<SI>(
        &self,
        used_vars: &mut UseMap,
        const_vars: &HashMap<Variable, Constant>,
        stmnt_iter: SI,
        block_id: ir::WeakBlockPtr,
    ) -> Vec<Statement>
    where
        SI: DoubleEndedIterator<Item = Statement>,
    {
        let mut result = Vec::new();

        for stmnt in stmnt_iter {
            match &stmnt {
                ir::Statement::Assignment { target, .. } if !used_vars.contains(target) => {
                    let used = stmnt.used_vars();

                    used.into_iter().for_each(|u| {
                        used_vars.decrement(&u);
                    });
                }
                ir::Statement::JumpTrue(var, target, _) if const_vars.contains_key(var) => {
                    let const_val = const_vars
                        .get(var)
                        .expect("We checked that the Variable is a constant");

                    // Determine if the Constant is false
                    let cond_res = matches!(
                        const_val,
                        Constant::I8(0)
                            | Constant::U8(0)
                            | Constant::I16(0)
                            | Constant::U16(0)
                            | Constant::I32(0)
                            | Constant::U32(0)
                            | Constant::I64(0)
                            | Constant::U64(0)
                    );

                    if !cond_res {
                        // If we dont know for sure that the Variable evaluates to false, then we
                        // should just keep the JumpTrue Statement
                        result.push(stmnt);
                    } else {
                        // The Variable is definetly a "false" Constant

                        // TODO
                        // Remove the current Block from the targets Predecessors because we now
                        // dont actually jump to it and are not an actual Predecessor anymore.
                        // However Im not really sure that we can just remove a Predecessor as
                        // there may be multiple Jumps from this Block to the other one?
                        //
                        // target.remove_predecessor(block_id.clone());
                        let _ = block_id;
                        let _ = target;
                    }
                }
                _ => result.push(stmnt),
            };
        }

        result
    }
}

impl Default for DeadCode {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for DeadCode {
    fn name(&self) -> String {
        "DeadCode".to_string()
    }

    fn pass_function(&self, ir: ir::FunctionDefinition) -> ir::FunctionDefinition {
        let mut used_vars = UseMap::new();
        let mut const_vars = HashMap::new();

        let graph = ir.to_directed_graph();
        graph
            .chain_iter()
            .flatten()
            .flat_map(|block| block.get_statements())
            // Add all the Used-Variables
            .inspect(|stmnt| {
                for tmp in stmnt.used_vars() {
                    used_vars.increment(tmp);
                }
            })
            // Add all the Const-Variables
            .inspect(|stmnt| {
                if let ir::Statement::Assignment {
                    target,
                    value: ir::Value::Constant(con),
                } = stmnt
                {
                    const_vars.insert(target.clone(), con.clone());
                }
            })
            // This is only used for running the Iterator to completion
            .count();

        for block in graph.chain_iter().flatten() {
            let statements = block.get_statements();

            let n_statments: Vec<_> = self.filter_stmnts(
                &mut used_vars,
                &const_vars,
                statements.into_iter(),
                block.weak_ptr(),
            );

            block.set_statements(n_statments);
        }

        ir
    }
}

#[cfg(test)]
mod tests {
    use ir::{BasicBlock, Constant, Statement, Variable};

    use super::*;

    #[test]
    fn redundant_assign() {
        let x0_var = Variable::new("x", ir::Type::I64);
        let x1_var = x0_var.next_gen();

        let ir_block = BasicBlock::new(
            vec![],
            vec![
                Statement::Assignment {
                    target: x0_var,
                    value: ir::Value::Constant(Constant::I64(13)),
                },
                Statement::Assignment {
                    target: x1_var.clone(),
                    value: ir::Value::Constant(Constant::I64(23)),
                },
                Statement::Return(Some(x1_var.clone())),
            ],
        );

        let func_def = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: ir::Type::U8,
            block: ir_block,
        };

        let expected_block = BasicBlock::new(
            vec![],
            vec![
                Statement::Assignment {
                    target: x1_var.clone(),
                    value: ir::Value::Constant(Constant::I64(23)),
                },
                Statement::Return(Some(x1_var)),
            ],
        );
        let expected_def = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: ir::Type::U8,
            block: expected_block,
        };

        let op_pass = DeadCode::new();

        let result_def = op_pass.pass_function(func_def);

        assert_eq!(expected_def, result_def);
    }
}
