use std::sync::Arc;

use ir::{BasicBlock, FunctionDefinition, Program, Statement, Type, Value, Variable};

use crate::{AAssignTarget, AScope, AStatement, FunctionDeclaration, AAST};

mod block;
pub use block::*;

mod expression;

pub fn convert(ast: AAST) -> Program {
    let global_block = BasicBlock::initial(vec![]);

    let mut functions = Vec::new();
    for (name, (func_dec, func_scope)) in ast.global_scope.0.function_definitions {
        let return_ty = Type::Void;

        let args = {
            let mut tmp = Vec::new();

            for arg in func_dec.arguments.iter() {
                let name = arg.data.name.0.data.clone();
                let ty = arg.data.ty.clone().to_ir();

                tmp.push((name, ty));
            }

            tmp
        };

        let func_block = convert_function(&global_block, name, func_dec, func_scope);
        dbg!(&func_block);

        functions.push(FunctionDefinition {
            arguments: args,
            return_ty,
            block: func_block,
        })
    }

    Program {
        global: global_block,
        functions,
    }
}

fn convert_function(
    global: &Arc<BasicBlock>,
    name: String,
    func_dec: FunctionDeclaration,
    inner_scope: AScope,
) -> Arc<BasicBlock> {
    dbg!(&name, &func_dec, &inner_scope);

    // Put the Arguments into the first basic Block and then place a Jump as the last Statement
    // that will jump to the actual function code

    let arg_statements = {
        let mut tmp = Vec::new();

        for tmp_arg in func_dec.arguments.iter() {
            let var_data = &tmp_arg.data;
            let var_ty = tmp_arg.data.ty.clone().to_ir();
            let var = Variable::new(&var_data.name.0.data, 0, var_ty);

            tmp.push(Statement::Assignment {
                target: var,
                value: Value::Unknown,
            });
        }

        tmp
    };

    let global_weak = Arc::downgrade(global);
    let head_block = BasicBlock::new(vec![global_weak], arg_statements);

    let head_weak = Arc::downgrade(&head_block);
    let func_block = BasicBlock::new(vec![head_weak], vec![]);
    convert_scope(inner_scope, &func_block);

    // Update Head-Blocks last Jump to the next
    head_block.add_statement(Statement::Jump(func_block));

    head_block
}

pub fn convert_scope(scope: AScope, block: &Arc<BasicBlock>) {
    for tmp_stmnt in scope.statements {
        dbg!(&tmp_stmnt);
        match tmp_stmnt {
            AStatement::Assignment { target, value } => {
                dbg!(&target, &value);

                let next_var = match target {
                    AAssignTarget::Variable { ident, ty_info } => {
                        dbg!(&ident, &ty_info);

                        let var_name = ident.0.data;
                        match block.definition(&var_name) {
                            Some(mut var) => {
                                dbg!(&var);

                                var.next_generation();
                                var
                            }
                            None => {
                                let target_ty = ty_info.data.to_ir();

                                Variable::new(var_name.clone(), 0, target_ty)
                            }
                        }
                    }
                    other => {
                        dbg!(&other);

                        todo!("Unknown Assign Target");
                    }
                };
                dbg!(&next_var);

                let value_exp = value.to_ir(&block);

                block.add_statement(Statement::Assignment {
                    target: next_var,
                    value: value_exp,
                });
            }
            AStatement::Expression(exp) => {
                dbg!(&exp);

                todo!("Parse Single  Expression");
            }
            AStatement::Return { value } => {
                dbg!(&value);

                let ret_stmnt = match value {
                    Some(val) => {
                        dbg!(&val);

                        todo!("")
                    }
                    None => Statement::Return(None),
                };

                block.add_statement(ret_stmnt);
            }
            other => {
                dbg!(&other);
            }
        };
    }
}
