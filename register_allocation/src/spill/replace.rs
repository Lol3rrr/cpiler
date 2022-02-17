pub fn expression_replace(
    exp: ir::Expression,
    to_replace: &ir::Variable,
    replacement: &ir::Variable,
) -> ir::Expression {
    match exp {
        ir::Expression::UnaryOp { base, op } => ir::Expression::UnaryOp {
            op,
            base: operand_replace(base, to_replace, replacement),
        },
        ir::Expression::BinaryOp { op, left, right } => ir::Expression::BinaryOp {
            op,
            left: operand_replace(left, to_replace, replacement),
            right: operand_replace(right, to_replace, replacement),
        },
        ir::Expression::FunctionCall {
            name,
            arguments,
            return_ty,
        } => ir::Expression::FunctionCall {
            name,
            arguments: arguments
                .into_iter()
                .map(|a| operand_replace(a, to_replace, replacement))
                .collect(),
            return_ty,
        },
        ir::Expression::ReadMemory { address, read_ty } => ir::Expression::ReadMemory {
            address: operand_replace(address, to_replace, replacement),
            read_ty,
        },
        ir::Expression::ReadGlobalVariable { name } => ir::Expression::ReadGlobalVariable { name },
        ir::Expression::StackAlloc { size, alignment } => {
            ir::Expression::StackAlloc { size, alignment }
        }
        ir::Expression::Cast { base, target } => ir::Expression::Cast {
            target,
            base: operand_replace(base, to_replace, replacement),
        },
        ir::Expression::AdressOf { base } => ir::Expression::AdressOf {
            base: operand_replace(base, to_replace, replacement),
        },
    }
}

pub fn value_replace(
    val: ir::Value,
    to_replace: &ir::Variable,
    replacement: &ir::Variable,
    replacement_block: &ir::BasicBlock,
) -> ir::Value {
    match val {
        ir::Value::Variable(var) if &var == to_replace => ir::Value::Variable(replacement.clone()),
        ir::Value::Variable(var) => ir::Value::Variable(var),
        ir::Value::Expression(exp) => {
            ir::Value::Expression(expression_replace(exp, to_replace, replacement))
        }
        ir::Value::Phi { sources } => {
            let n_sources: Vec<_> = sources
                .into_iter()
                .map(|mut e| {
                    if &e.var == to_replace {
                        e.var = replacement.clone();
                        e.block = replacement_block.weak_ptr();
                        e
                    } else {
                        e
                    }
                })
                .collect();
            ir::Value::Phi { sources: n_sources }
        }
        ir::Value::Constant(con) => ir::Value::Constant(con),
        ir::Value::Unknown => ir::Value::Unknown,
    }
}

pub fn operand_replace(
    op: ir::Operand,
    to_replace: &ir::Variable,
    replacement: &ir::Variable,
) -> ir::Operand {
    match op {
        ir::Operand::Constant(con) => ir::Operand::Constant(con),
        ir::Operand::Variable(var) if &var == to_replace => {
            ir::Operand::Variable(replacement.clone())
        }
        ir::Operand::Variable(var) => ir::Operand::Variable(var),
    }
}

pub fn statement(
    stmnt: ir::Statement,
    to_replace: &ir::Variable,
    replacement: &ir::Variable,
    replacement_block: &ir::BasicBlock,
) -> ir::Statement {
    match stmnt {
        ir::Statement::Assignment { target, value } => ir::Statement::Assignment {
            target,
            value: value_replace(value, to_replace, replacement, replacement_block),
        },
        ir::Statement::WriteMemory { target, value } => ir::Statement::WriteMemory {
            target: operand_replace(target, to_replace, replacement),
            value: value_replace(value, to_replace, replacement, replacement_block),
        },
        ir::Statement::Call { name, arguments } => {
            let n_args: Vec<_> = arguments
                .into_iter()
                .map(|a| operand_replace(a, to_replace, replacement))
                .collect();
            ir::Statement::Call {
                name,
                arguments: n_args,
            }
        }
        ir::Statement::SaveVariable { var } => {
            if &var == to_replace {
                ir::Statement::SaveVariable {
                    var: replacement.clone(),
                }
            } else {
                ir::Statement::SaveVariable { var }
            }
        }
        ir::Statement::SaveGlobalVariable { var } => {
            if &var == to_replace {
                ir::Statement::SaveGlobalVariable {
                    var: replacement.clone(),
                }
            } else {
                ir::Statement::SaveGlobalVariable { var }
            }
        }
        ir::Statement::InlineAsm {
            template,
            inputs,
            output,
        } => {
            let n_output = match output {
                Some(var) if &var == to_replace => Some(replacement.clone()),
                og => og,
            };

            let n_inputs: Vec<_> = inputs
                .into_iter()
                .map(|v| {
                    if &v == to_replace {
                        replacement.clone()
                    } else {
                        v
                    }
                })
                .collect();

            ir::Statement::InlineAsm {
                template,
                inputs: n_inputs,
                output: n_output,
            }
        }
        ir::Statement::Return(var) => {
            let n_var = match var {
                Some(var) if &var == to_replace => Some(replacement.clone()),
                other => other,
            };

            ir::Statement::Return(n_var)
        }
        ir::Statement::Jump(target) => ir::Statement::Jump(target),
        ir::Statement::JumpTrue(var, target) => {
            if &var == to_replace {
                ir::Statement::JumpTrue(replacement.clone(), target)
            } else {
                ir::Statement::JumpTrue(var, target)
            }
        }
    }
}
