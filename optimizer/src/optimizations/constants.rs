use std::collections::HashMap;

use ir::{BinaryOp, Constant, Expression, Operand, Statement, Value, Variable};

use crate::OptimizationPass;

mod binary;

/// This is used for things like constant Propagation and Evaluation
#[derive(Debug)]
pub struct ConstantProp {}

impl ConstantProp {
    /// Creates a new Instance of this Pass
    pub fn new() -> Self {
        Self {}
    }

    fn const_eval(&self, stmnt: Statement, consts: &mut HashMap<Variable, Constant>) -> Statement {
        match stmnt {
            Statement::Assignment {
                target,
                value: Value::Constant(con),
            } => {
                consts.insert(target.clone(), con.clone());

                Statement::Assignment {
                    target,
                    value: Value::Constant(con),
                }
            }
            Statement::Assignment { target, value } => {
                let n_value = match value {
                    Value::Expression(exp) => match exp {
                        Expression::BinaryOp {
                            op: BinaryOp::Arith(arith_op),
                            left: Operand::Constant(left_con),
                            right: Operand::Constant(right_con),
                        } => binary::binary_arith_consts(arith_op, left_con, right_con),
                        Expression::BinaryOp {
                            op,
                            left: Operand::Variable(left_var),
                            right,
                        } if consts.contains_key(&left_var) => {
                            let con_replacement = consts.get(&left_var).unwrap().clone();

                            Value::Expression(Expression::BinaryOp {
                                op,
                                left: Operand::Constant(con_replacement),
                                right,
                            })
                        }
                        Expression::BinaryOp {
                            op,
                            left,
                            right: Operand::Variable(right_var),
                        } if consts.contains_key(&right_var) => {
                            let con_replacement = consts.get(&right_var).unwrap().clone();

                            Value::Expression(Expression::BinaryOp {
                                op,
                                left,
                                right: Operand::Constant(con_replacement),
                            })
                        }
                        Expression::Cast {
                            base: ir::Operand::Constant(con),
                            target,
                        } => match (con, target) {
                            (Constant::I64(b_val), ir::Type::I32) => {
                                Value::Constant(Constant::I32(b_val as i32))
                            }
                            (con, target) => Value::Expression(Expression::Cast {
                                base: ir::Operand::Constant(con),
                                target,
                            }),
                        },
                        other => Value::Expression(other),
                    },
                    other => other,
                };

                Statement::Assignment {
                    target,
                    value: n_value,
                }
            }
            other => other,
        }
    }
}

impl Default for ConstantProp {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for ConstantProp {
    fn name(&self) -> String {
        "ConstantProp".to_owned()
    }

    fn pass_function(&self, ir: ir::FunctionDefinition) -> ir::FunctionDefinition {
        for block in ir.block.block_iter() {
            let statements = block.get_statements();
            let mut const_vars: HashMap<Variable, Constant> = HashMap::new();

            let n_statements = statements
                .into_iter()
                .map(|s| self.const_eval(s, &mut const_vars))
                .collect();
            block.set_statements(n_statements);
        }

        ir
    }
}

#[cfg(test)]
mod tests {
    use ir::{BasicBlock, BinaryArithmeticOp, BinaryOp, Expression, Operand, Type};

    use super::*;

    #[test]
    fn constant_exp() {
        let block = BasicBlock::new(
            vec![],
            vec![
                Statement::Assignment {
                    target: Variable::tmp(1, Type::I64),
                    value: Value::Expression(Expression::BinaryOp {
                        op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                        left: Operand::Constant(Constant::I64(0)),
                        right: Operand::Constant(Constant::I64(13)),
                    }),
                },
                Statement::Assignment {
                    target: Variable::tmp(2, Type::I64),
                    value: Value::Expression(Expression::BinaryOp {
                        op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                        left: Operand::Constant(Constant::I64(0)),
                        right: Operand::Constant(Constant::I64(13)),
                    }),
                },
            ],
        );
        let ir_func = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: Type::Void,
            block,
        };

        let expected_block = BasicBlock::new(
            vec![],
            vec![
                Statement::Assignment {
                    target: Variable::tmp(1, Type::I64),
                    value: Value::Constant(Constant::I64(13)),
                },
                Statement::Assignment {
                    target: Variable::tmp(2, Type::I64),
                    value: Value::Constant(Constant::I64(0)),
                },
            ],
        );

        let pass = ConstantProp::new();
        let result = pass.pass_function(ir_func);
        dbg!(&result);

        assert_eq!(expected_block, result.block);
    }

    #[test]
    fn constant_var() {
        let block = BasicBlock::new(
            vec![],
            vec![
                Statement::Assignment {
                    target: Variable::tmp(1, Type::I64),
                    value: Value::Constant(Constant::I64(13)),
                },
                Statement::Assignment {
                    target: Variable::tmp(2, Type::I64),
                    value: Value::Expression(Expression::BinaryOp {
                        op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                        left: Operand::Variable(Variable::tmp(1, Type::I64)),
                        right: Operand::Constant(Constant::I64(13)),
                    }),
                },
            ],
        );
        let ir_func = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: Type::Void,
            block,
        };

        let expected_block = BasicBlock::new(
            vec![],
            vec![
                Statement::Assignment {
                    target: Variable::tmp(1, Type::I64),
                    value: Value::Constant(Constant::I64(13)),
                },
                Statement::Assignment {
                    target: Variable::tmp(2, Type::I64),
                    value: Value::Expression(Expression::BinaryOp {
                        op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                        left: Operand::Constant(Constant::I64(13)),
                        right: Operand::Constant(Constant::I64(13)),
                    }),
                },
            ],
        );

        let pass = ConstantProp::new();
        let result = pass.pass_function(ir_func);
        dbg!(&result);

        assert_eq!(expected_block, result.block);
    }
}
