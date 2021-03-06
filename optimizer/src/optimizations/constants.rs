use std::collections::HashMap;

use ir::{BinaryOp, Constant, Expression, Operand, Statement, UnaryOp, Value, Variable};

use crate::OptimizationPass;

mod binary;
mod unary;

/// This is used for things like constant Propagation and Evaluation
#[derive(Debug)]
pub struct ConstantProp {}

impl ConstantProp {
    /// Creates a new Instance of this Pass
    pub fn new() -> Self {
        Self {}
    }

    fn eval_expression(&self, exp: Expression, consts: &mut HashMap<Variable, Constant>) -> Value {
        match exp {
            Expression::BinaryOp {
                op: BinaryOp::Arith(arith_op),
                left: Operand::Constant(left_con),
                right: Operand::Constant(right_con),
            } => binary::binary_arith_consts(arith_op, left_con, right_con),
            Expression::BinaryOp { op, left, right } => {
                let n_left = match left {
                    Operand::Constant(con) => Operand::Constant(con),
                    Operand::Variable(var) => match consts.get(&var) {
                        Some(con) => Operand::Constant(con.clone()),
                        None => Operand::Variable(var),
                    },
                };
                let n_right = match right {
                    Operand::Constant(con) => Operand::Constant(con),
                    Operand::Variable(var) => match consts.get(&var) {
                        Some(con) => Operand::Constant(con.clone()),
                        None => Operand::Variable(var),
                    },
                };

                Value::Expression(Expression::BinaryOp {
                    op,
                    left: n_left,
                    right: n_right,
                })
            }
            Expression::UnaryOp {
                op,
                base: Operand::Constant(con_base),
            } => match op {
                UnaryOp::Arith(arith_op) => unary::arith_con(arith_op, con_base),
                _ => Value::Expression(Expression::UnaryOp {
                    op,
                    base: Operand::Constant(con_base),
                }),
            },
            Expression::UnaryOp { op, base } => {
                let n_base = match base {
                    Operand::Constant(con) => Operand::Constant(con),
                    Operand::Variable(var) => match consts.get(&var) {
                        Some(con) => Operand::Constant(con.clone()),
                        None => Operand::Variable(var),
                    },
                };

                Value::Expression(Expression::UnaryOp { op, base: n_base })
            }
            Expression::Cast {
                base: ir::Operand::Constant(con),
                target,
            } => match (con, target) {
                (Constant::I64(b_val), ir::Type::I32) => {
                    Value::Constant(Constant::I32(b_val as i32))
                }
                (Constant::I64(b_val), ir::Type::I64) => Value::Constant(Constant::I64(b_val)),
                (Constant::I64(b_val), ir::Type::I8) => Value::Constant(Constant::I8(b_val as i8)),
                (con, target) => {
                    dbg!(&con, &target);

                    Value::Expression(Expression::Cast {
                        base: ir::Operand::Constant(con),
                        target,
                    })
                }
            },
            other => Value::Expression(other),
        }
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
                    Value::Expression(exp) => self.eval_expression(exp, consts),
                    Value::Phi { sources } => {
                        let n_sources: Vec<_> = sources
                            .into_iter()
                            .filter(|s| s.block.upgrade().is_some())
                            .collect();

                        Value::Phi { sources: n_sources }
                    }
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
        let mut const_vars: HashMap<Variable, Constant> = HashMap::new();

        let graph = ir.to_directed_graph();
        for block in graph.chain_iter().flatten() {
            let statements = block.get_statements();

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

    #[test]
    fn in_seperate_blocks() {
        let c_var = Variable::new("testing", Type::U32);
        let c_value = Constant::U32(13);
        let t1 = Variable::new("other", Type::U32);

        let start_block = BasicBlock::new(
            vec![],
            vec![ir::Statement::Assignment {
                target: c_var.clone(),
                value: ir::Value::Constant(c_value.clone()),
            }],
        );
        let second_block = BasicBlock::new(
            vec![start_block.weak_ptr()],
            vec![ir::Statement::Assignment {
                target: t1.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Variable(c_var.clone()),
                    right: Operand::Constant(Constant::I64(13)),
                }),
            }],
        );
        start_block.add_statement(Statement::Jump(second_block, ir::JumpMetadata::Linear));

        let ir_func = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: Type::Void,
            block: start_block,
        };

        let expected_first = BasicBlock::new(
            vec![],
            vec![ir::Statement::Assignment {
                target: c_var,
                value: ir::Value::Constant(c_value.clone()),
            }],
        );
        let expected_second = BasicBlock::new(
            vec![expected_first.weak_ptr()],
            vec![ir::Statement::Assignment {
                target: t1,
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Constant(c_value),
                    right: Operand::Constant(Constant::I64(13)),
                }),
            }],
        );
        expected_first.add_statement(Statement::Jump(expected_second, ir::JumpMetadata::Linear));

        let expected_func = ir::FunctionDefinition {
            name: "test".to_string(),
            arguments: vec![],
            return_ty: Type::Void,
            block: expected_first,
        };

        let pass = ConstantProp::new();
        let result = pass.pass_function(ir_func);
        dbg!(&result);

        assert_eq!(expected_func, result);
    }
}
