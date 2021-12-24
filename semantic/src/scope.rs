use std::collections::HashMap;

use ir::{BasicBlock, Statement, Type, Variable};
use syntax::Scope;

use crate::{
    conversion::ConvertContext, AAssignTarget, AExpression, AStatement, AType, FunctionDeclaration,
    SemanticError,
};

mod state;
pub use state::*;

#[derive(Debug, PartialEq)]
pub struct ARootScope(pub AScope);
impl ARootScope {
    pub fn parse(scope: Scope) -> Result<Self, SemanticError> {
        let initial_state = ParseState::new();

        let scope = AScope::parse(&initial_state, scope)?;
        Ok(Self(scope))
    }
}

#[derive(Debug, PartialEq)]
pub struct AScope {
    pub statements: Vec<AStatement>,
    pub function_definitions: HashMap<String, (FunctionDeclaration, AScope)>,
}

impl AScope {
    pub fn parse(external_state: &ParseState, scope: Scope) -> Result<Self, SemanticError> {
        let mut current = ParseState::based(external_state);
        dbg!(&current);

        let mut statements: Vec<AStatement> = Vec::new();

        for statement in scope.statements {
            if let Some(tmp) = AStatement::parse(statement, &mut current)? {
                dbg!(&tmp);

                statements.push(tmp);
            }
        }

        let func_definitions = current.destructure();

        Ok(Self {
            statements,
            function_definitions: func_definitions,
        })
    }

    pub fn from_parse_state(state: ParseState, statements: Vec<AStatement>) -> Self {
        let func_definitions = state.destructure();

        Self {
            statements,
            function_definitions: func_definitions,
        }
    }

    /// This will convert the current Scope into the IR used for in the Rest of the Compiler.
    /// The given Block will be used as the starting Point but as there may be multiple basic
    /// blocks produced as a result of this conversion (like with different control-flows)
    /// so the final BasicBlock will be returned and can then be used as a starting Point
    /// for the next Statements following this Scope
    pub fn to_ir(self, block: &BasicBlock, ctx: &ConvertContext) -> BasicBlock {
        let mut block = block.clone();

        for tmp_stmnt in self.statements {
            match tmp_stmnt {
                AStatement::SubScope { inner } => {
                    let sub_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);
                    block.add_statement(Statement::Jump(sub_block.clone()));

                    let end_sub_block = inner.to_ir(&sub_block, ctx);
                    let following_block = BasicBlock::new(vec![end_sub_block.weak_ptr()], vec![]);
                    end_sub_block.add_statement(Statement::Jump(following_block.clone()));

                    block = following_block;
                }
                AStatement::DeclareVar { name, ty: raw_ty } => {
                    dbg!(&name, &raw_ty);
                    let ty = raw_ty.ty();

                    let target_name = name.0.data;
                    if block.definition(&target_name, &|| 0).is_some() {
                        panic!("");
                    }

                    match ty {
                        AType::Array(arr) => {
                            let arr_length = arr.size.unwrap();
                            let alignment = arr.ty.alignment(ctx.arch()) as usize;
                            let size = arr_length * arr.ty.byte_size(ctx.arch()) as usize;

                            let ir_ty = arr.ty.to_ir();
                            let target_var =
                                Variable::new(target_name, ir::Type::Pointer(Box::new(ir_ty)));
                            dbg!(&target_var);

                            let reserve_exp = ir::Expression::StackAlloc { size, alignment };
                            dbg!(&reserve_exp);

                            block.add_statement(Statement::Assignment {
                                target: target_var,
                                value: ir::Value::Expression(reserve_exp),
                            });
                        }
                        AType::Struct(struct_def) => {
                            dbg!(&struct_def);

                            let size = struct_def.entire_size(ctx.arch());
                            let alignment = struct_def.alignment(ctx.arch());
                            dbg!(&size, &alignment);

                            let ir_ty = ir::Type::Pointer(Box::new(ir::Type::Void));
                            let target_var = Variable::new(target_name, ir_ty);
                            dbg!(&target_var);

                            let reserve_exp = ir::Expression::StackAlloc { size, alignment };
                            dbg!(&reserve_exp);

                            block.add_statement(Statement::Assignment {
                                target: target_var,
                                value: ir::Value::Expression(reserve_exp),
                            });
                        }
                        AType::Primitve(_) => {
                            let ir_type = ty.to_ir();
                            dbg!(&ir_type);

                            let var = Variable::new(target_name, ir_type);
                            block.add_statement(Statement::Assignment {
                                target: var,
                                value: ir::Value::Unknown,
                            });
                        }
                        other => {
                            dbg!(&other);

                            todo!("");
                        }
                    };
                }
                AStatement::Assignment { target, value } => {
                    dbg!(&target, &value);

                    match target {
                        AAssignTarget::Variable { ident, ty_info } => {
                            dbg!(&ident, &ty_info);

                            let var_name = ident.0.data;
                            let next_var = match block.definition(&var_name, &|| ctx.next_tmp()) {
                                Some(var) => var.next_gen(),
                                None => {
                                    let target_ty = ty_info.data.to_ir();

                                    ir::Variable::new(var_name.clone(), target_ty)
                                }
                            };

                            dbg!(&next_var);

                            let value_exp = value.to_ir(&block, ctx);
                            let target_meta = value_exp.assign_meta(&next_var);
                            let target_var = next_var.set_meta(target_meta);

                            block.add_statement(ir::Statement::Assignment {
                                target: target_var,
                                value: value_exp,
                            });
                        }
                        AAssignTarget::Deref { exp, .. } => {
                            let address_value = exp.to_ir(&block, ctx);
                            dbg!(&address_value);

                            let target_oper =
                                AExpression::val_to_operand(address_value, &block, ctx);
                            dbg!(&target_oper);

                            let value_exp = value.to_ir(&block, ctx);
                            dbg!(&value_exp);

                            match &target_oper {
                                ir::Operand::Variable(target_var) => {
                                    match target_var.meta() {
                                        ir::VariableMetadata::VarPointer { var } => {
                                            let next_var = var.next_gen();
                                            let target_meta = value_exp.assign_meta(&next_var);
                                            let target_var = next_var.set_meta(target_meta);

                                            block.add_statement(Statement::Assignment {
                                                target: target_var,
                                                value: value_exp.clone(),
                                            });
                                        }
                                        _ => {}
                                    };
                                }
                                _ => {}
                            };

                            block.add_statement(ir::Statement::WriteMemory {
                                target: target_oper,
                                value: value_exp,
                            });
                        }
                        AAssignTarget::ArrayAccess(target) => {
                            dbg!(&target);
                            let target_exp = target.to_exp(&block, ctx);
                            dbg!(&target_exp);

                            let target_value = ir::Value::Expression(target_exp);
                            let target_oper =
                                AExpression::val_to_operand(target_value, &block, ctx);
                            dbg!(&target_oper);

                            let value_exp = value.to_ir(&block, ctx);
                            dbg!(&value_exp);

                            block.add_statement(ir::Statement::WriteMemory {
                                target: target_oper,
                                value: value_exp,
                            });
                        }
                        AAssignTarget::StructField(target) => {
                            dbg!(&target);
                            let target_exp = target.to_exp(&block, ctx);
                            dbg!(&target_exp);

                            let target_value = ir::Value::Expression(target_exp);
                            let target_oper =
                                AExpression::val_to_operand(target_value, &block, ctx);
                            dbg!(&target_oper);

                            let value = value.to_ir(&block, ctx);
                            dbg!(&value);

                            block.add_statement(ir::Statement::WriteMemory {
                                target: target_oper,
                                value,
                            });
                        }
                    };
                }
                AStatement::Expression(exp) => {
                    dbg!(&exp);

                    // TODO
                    // No idea how I might go about doing this

                    match exp {
                        AExpression::FunctionCall(call) => {
                            call.to_standalone_ir(&block, ctx);
                        }
                        other => {
                            dbg!(&other);

                            todo!("Unknown Standalone Expression")
                        }
                    };
                }
                AStatement::Return { value } => {
                    dbg!(&value);

                    let ret_value = match value {
                        Some(raw_ret) => {
                            dbg!(&raw_ret);
                            let raw_ty = raw_ret.result_type();
                            let target_ty = raw_ty.to_ir();

                            let ret_exp = raw_ret.to_ir(&block, ctx);
                            dbg!(&ret_exp);

                            let ret_var = Variable::tmp(ctx.next_tmp(), target_ty);
                            block.add_statement(Statement::Assignment {
                                target: ret_var.clone(),
                                value: ret_exp,
                            });

                            Some(ret_var)
                        }
                        None => None,
                    };

                    block.add_statement(ir::Statement::Return(ret_value));
                }
                AStatement::If {
                    body,
                    condition,
                    else_,
                } => {
                    dbg!(&body, &condition);

                    let cond_value = condition.to_ir(&block, ctx);
                    let cond_var = Variable::tmp(ctx.next_tmp(), Type::I64);

                    let cond_statement = ir::Statement::Assignment {
                        target: cond_var.clone(),
                        value: cond_value,
                    };
                    block.add_statement(cond_statement);

                    // The final resulting Block we reach after the If-Statement is complete
                    let end_block = BasicBlock::new(vec![], vec![]);

                    // The Block for the inner Scope of the If-Statement if true
                    let true_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);
                    let end_true_body = body.to_ir(&true_block, ctx);
                    end_true_body.add_statement(Statement::Jump(end_block.clone()));
                    end_block.add_predecessor(end_true_body.weak_ptr());

                    block.add_statement(Statement::JumpTrue(cond_var.clone(), true_block.clone()));

                    if let Some(else_) = else_ {
                        let false_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);
                        let end_false_block = else_.to_ir(&false_block, ctx);
                        end_false_block.add_statement(Statement::Jump(end_block.clone()));
                        block.add_statement(Statement::Jump(end_false_block.clone()));
                        end_block.add_predecessor(end_false_block.weak_ptr());
                    } else {
                        // Jump to the end Block directly
                        block.add_statement(Statement::Jump(end_block.clone()));
                        end_block.add_predecessor(block.weak_ptr());
                    }

                    block = end_block;
                }
                AStatement::WhileLoop { condition, body } => {
                    dbg!(&condition, &body);

                    let start_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);
                    let inner_block = BasicBlock::new(vec![start_block.weak_ptr()], vec![]);
                    let end_block = BasicBlock::new(vec![start_block.weak_ptr()], vec![]);

                    // Generate the first iteration of the start Block
                    {
                        let cond_value = condition.to_ir(&start_block, ctx);
                        let cond_var = Variable::tmp(ctx.next_tmp(), Type::I64);

                        let cond_statement = ir::Statement::Assignment {
                            target: cond_var.clone(),
                            value: cond_value,
                        };
                        start_block.add_statement(cond_statement);

                        start_block
                            .add_statement(Statement::JumpTrue(cond_var, inner_block.clone()));
                        start_block.add_statement(Statement::Jump(end_block.clone()));
                    }

                    // Generate the inner Part of the Loop
                    {
                        let loop_ctx = ctx.with_loop(start_block.clone(), end_block.clone());

                        let inner_end_block = body.to_ir(&inner_block, &loop_ctx);
                        inner_end_block.add_statement(Statement::Jump(start_block.clone()));
                        start_block.add_predecessor(inner_end_block.weak_ptr());
                    }

                    // TODO
                    // Regenerate the condition of the Loop

                    block.add_statement(Statement::Jump(start_block));
                    block = end_block;
                }
                AStatement::ForLoop {
                    condition,
                    body,
                    updates,
                } => {
                    dbg!(&condition, &body, &updates);

                    let used_vars_in_cond = condition.used_variables();
                    for tmp_var in used_vars_in_cond {
                        dbg!(&tmp_var);
                    }

                    // The Block containing the Condition
                    let condition_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);

                    // The starting Block of the internal Scope
                    let content_start_block =
                        BasicBlock::new(vec![condition_block.weak_ptr()], vec![]);

                    // The Block responsible for updating some Variant for the Loop
                    let update_block = BasicBlock::new(vec![], vec![]);
                    condition_block.add_predecessor(update_block.weak_ptr());

                    // The first Block after the actual Loop
                    let end_block = BasicBlock::new(vec![condition_block.weak_ptr()], vec![]);

                    todo!("Generate IR for For-Loop");
                }
                AStatement::Break => {
                    let loop_end_block = match ctx.get_loop_end() {
                        Some(b) => b,
                        None => panic!("Break outside of Loop"),
                    };

                    loop_end_block.add_predecessor(block.weak_ptr());
                    block.add_statement(Statement::Jump(loop_end_block.clone()));
                }
                AStatement::Continue => {
                    let loop_start_block = match ctx.get_loop_start() {
                        Some(b) => b,
                        None => panic!("Continue outside of loop"),
                    };

                    loop_start_block.add_predecessor(block.weak_ptr());
                    block.add_statement(Statement::Jump(loop_start_block.clone()));
                }
            };
        }

        block
    }
}
