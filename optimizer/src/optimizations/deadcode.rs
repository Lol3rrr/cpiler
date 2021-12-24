use std::collections::HashSet;

use crate::Optimization;

/// Performs basic DeadCode Elimination
pub struct DeadCode {}

impl DeadCode {
    /// Creates a new DeadCode Pass Instance
    pub fn new() -> Self {
        Self {}
    }
}

impl Optimization for DeadCode {
    fn name(&self) -> String {
        "DeadCode".to_string()
    }

    fn pass_function(&self, ir: ir::FunctionDefinition) -> ir::FunctionDefinition {
        let mut used_vars = HashSet::new();

        for block in ir.block.block_iter() {
            let statements = block.get_statements();

            for tmp_stmnt in statements {
                used_vars.extend(tmp_stmnt.used_vars());
            }
        }

        dbg!(&used_vars);

        for block in ir.block.block_iter() {
            let statements = block.get_statements();
            dbg!(&statements);

            let n_statments: Vec<_> = statements
                .into_iter()
                .filter(|stmnt| match stmnt {
                    ir::Statement::Assignment { target, .. } if !used_vars.contains(target) => {
                        false
                    }
                    _ => true,
                })
                .collect();
            dbg!(&n_statments);

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
                    target: x0_var.clone(),
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
                Statement::Return(Some(x1_var.clone())),
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
