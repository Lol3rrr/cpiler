use std::collections::HashSet;

fn expression_replace(
    exp: ir::Expression,
    to_replace: &ir::Variable,
    replacement: &ir::Variable,
) -> ir::Expression {
    match exp {
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
        other => {
            dbg!(&other);
            todo!()
        }
    }
}

fn value_replace(
    val: ir::Value,
    to_replace: &ir::Variable,
    replacement: &ir::Variable,
) -> ir::Value {
    match val {
        ir::Value::Variable(var) if &var == to_replace => ir::Value::Variable(replacement.clone()),
        ir::Value::Variable(var) => ir::Value::Variable(var),
        ir::Value::Expression(exp) => {
            ir::Value::Expression(expression_replace(exp, to_replace, replacement))
        }
        ir::Value::Phi { sources } => {
            todo!()
        }
        ir::Value::Constant(con) => ir::Value::Constant(con),
        ir::Value::Unknown => ir::Value::Unknown,
    }
}

fn operand_replace(
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

fn replace_statements<SI>(
    mut statements: SI,
    to_replace: &ir::Variable,
    replacement: &ir::Variable,
) -> Vec<ir::Statement>
where
    SI: Iterator<Item = ir::Statement>,
{
    let mut result = Vec::new();

    for s in statements.by_ref() {
        let tmp = match s {
            ir::Statement::Assignment { target, value } if &target == to_replace => {
                result.push(ir::Statement::Assignment { target, value });
                continue;
            }
            ir::Statement::Assignment { target, value } => ir::Statement::Assignment {
                target,
                value: value_replace(value, to_replace, replacement),
            },
            ir::Statement::WriteMemory { target, value } => ir::Statement::WriteMemory {
                target: operand_replace(target, to_replace, replacement),
                value: value_replace(value, to_replace, replacement),
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
            ir::Statement::SaveGlobalVariable { name } => {
                ir::Statement::SaveGlobalVariable { name }
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
        };

        result.push(tmp);
    }

    result.extend(statements);

    result
}

fn spill_in_block(
    block: ir::BasicBlock,
    to_replace: &ir::Variable,
    replacement: &ir::Variable,
    visited: &mut HashSet<*const ir::InnerBlock>,
) {
    todo!()
}

/// To spill a Variable we will in Principle create a new Generation of the Variable and
/// only assign it just before the spilled Variable would be first used after the Spill.
/// From that Point on we then need to replace all the uses of the Spilled Variable with the new
/// Generation
///
/// We need to consider three major situations:
/// * There is no Control-Flow in the Program
/// * There are branches in the Program but no loops
/// * There are loops in the Program
pub fn spill_variable(spill_var: ir::Variable, start_block: ir::BasicBlock, start_index: usize) {
    let replacement = spill_var.next_gen();

    let initial_statements = start_block.get_statements();
    let mut initial_iter = initial_statements.into_iter();

    let mut resulting: Vec<_> = initial_iter.by_ref().take(start_index).collect();
    resulting.extend(replace_statements(initial_iter, &spill_var, &replacement));

    let first_use_index = resulting
        .iter()
        .enumerate()
        .find(|(_, s)| s.used_vars().contains(&replacement))
        .map(|(i, _)| i);

    match first_use_index {
        Some(index) => {
            resulting.insert(
                index,
                ir::Statement::Assignment {
                    target: replacement.clone(),
                    value: ir::Value::Unknown,
                },
            );
        }
        None => {
            resulting.push(ir::Statement::Assignment {
                target: replacement.clone(),
                value: ir::Value::Unknown,
            });
        }
    };

    start_block.set_statements(resulting);

    let mut visited = HashSet::new();

    for (_, succ) in start_block.successors() {
        spill_in_block(succ, &spill_var, &replacement, &mut visited);
    }
}
