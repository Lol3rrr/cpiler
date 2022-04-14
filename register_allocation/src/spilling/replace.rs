fn replace_operand(oper: &mut ir::Operand, previous: &ir::Variable, n_var: &ir::Variable) {
    match oper {
        ir::Operand::Variable(var) if var == previous => {
            *var = n_var.clone();
        }
        ir::Operand::Variable(_) => {}
        ir::Operand::Constant(_) => {}
    };
}

fn replace_expression(exp: &mut ir::Expression, previous: &ir::Variable, n_var: &ir::Variable) {
    match exp {
        ir::Expression::AdressOf { base } => {
            replace_operand(base, previous, n_var);
        }
        ir::Expression::BinaryOp { left, right, .. } => {
            replace_operand(left, previous, n_var);
            replace_operand(right, previous, n_var);
        }
        ir::Expression::Cast { base, .. } => {
            replace_operand(base, previous, n_var);
        }
        ir::Expression::FunctionCall { arguments, .. } => {
            for arg in arguments {
                replace_operand(arg, previous, n_var);
            }
        }
        ir::Expression::UnaryOp { base, .. } => {
            replace_operand(base, previous, n_var);
        }
        ir::Expression::ReadGlobalVariable { .. } => {}
        ir::Expression::StackAlloc { .. } => {}
        ir::Expression::ReadMemory { address, .. } => {
            replace_operand(address, previous, n_var);
        }
    };
}

pub fn replace_used_variables(
    stmnt: &mut ir::Statement,
    previous: &ir::Variable,
    n_var: &ir::Variable,
) {
    match stmnt {
        ir::Statement::SaveVariable { var } if var == previous => {
            *var = n_var.clone();
        }
        ir::Statement::SaveVariable { .. } => {}
        ir::Statement::Assignment { value, .. } => {
            match value {
                ir::Value::Expression(exp) => {
                    replace_expression(exp, previous, n_var);
                }
                ir::Value::Variable(var) if var == previous => {
                    *var = n_var.clone();
                }
                ir::Value::Variable(_) => {}
                ir::Value::Phi { sources } => {
                    for src in sources.iter_mut() {
                        if &src.var == previous {
                            src.var = n_var.clone();
                        }
                    }

                    let first = sources.first().expect("");
                    if sources.iter().all(|v| first.var == v.var) {
                        let var = first.var.clone();

                        *value = ir::Value::Variable(var);
                    }
                }
                ir::Value::Constant(_) => {}
                ir::Value::Unknown => {}
            };
        }
        ir::Statement::Return(Some(var)) if var == previous => {
            *var = n_var.clone();
        }
        ir::Statement::Return(_) => {}
        ir::Statement::Jump(_, _) => {}
        ir::Statement::JumpTrue(var, _, _) if var == previous => {
            *var = n_var.clone();
        }
        ir::Statement::JumpTrue(_, _, _) => {}
        ir::Statement::SaveGlobalVariable { var } if var == previous => {
            *var = n_var.clone();
        }
        ir::Statement::SaveGlobalVariable { .. } => {}
        ir::Statement::WriteMemory { target, value } => {
            replace_operand(target, previous, n_var);
            replace_operand(value, previous, n_var);
        }
        ir::Statement::Call { arguments, .. } => {
            for arg in arguments.iter_mut() {
                replace_operand(arg, previous, n_var);
            }
        }
        ir::Statement::InlineAsm { .. } => {
            todo!("Replace inlined asm")
        }
    };
}
