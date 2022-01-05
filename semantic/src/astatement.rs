use general::SpanData;
use ir::BasicBlock;
use syntax::{AssignTarget, FunctionHead, Identifier, Statement};

use crate::{
    atype, conversion::ConvertContext, AExpression, AFunctionArg, APrimitive, AScope, AType,
    FunctionDeclaration, ParseState, SemanticError, UnaryArithmeticOp, UnaryOperator,
};

mod target;
pub use target::*;

#[derive(Debug, PartialEq, Clone)]
pub enum AStatement {
    DeclareVar {
        name: String,
        src: Identifier,
        ty: AType,
    },
    Assignment {
        target: AAssignTarget,
        value: AExpression,
    },
    Expression(AExpression),
    WhileLoop {
        condition: AExpression,
        body: AScope,
    },
    ForLoop {
        condition: AExpression,
        updates: Vec<Self>,
        body: AScope,
    },
    Break,
    Continue,
    If {
        condition: AExpression,
        body: AScope,
        else_: Option<AScope>,
    },
    Return {
        value: Option<AExpression>,
    },
    SubScope {
        inner: AScope,
    },
}

impl AStatement {
    pub fn parse(
        raw: Statement,
        parse_state: &mut ParseState,
    ) -> Result<Option<Self>, SemanticError> {
        match raw {
            Statement::TypeDef { name, base_type } => {
                let target_ty =
                    AType::parse_typedef(base_type, parse_state.type_defs(), parse_state)?;

                parse_state.mut_type_defs().add_definition(name, target_ty);

                Ok(None)
            }
            Statement::StructDefinition {
                name,
                members,
                definition,
            } => {
                let ty =
                    AType::parse_struct(members, definition, parse_state.type_defs(), parse_state)?;

                parse_state.mut_type_defs().add_definition(name, ty);
                Ok(None)
            }
            Statement::FunctionDeclaration(FunctionHead {
                name,
                r_type,
                arguments,
                var_args,
            }) => {
                if parse_state.is_declared(&name) {
                    panic!("Redefinition Error");
                }

                let r_ty = AType::parse(r_type, parse_state.type_defs(), parse_state)?;

                let arguments = {
                    let mut tmp = Vec::new();
                    for arg in arguments {
                        let tmp_ty =
                            AType::parse(arg.data.ty, parse_state.type_defs(), parse_state)?;
                        let int_name = ParseState::unique_var_name(&arg.data.name, &arg.span);
                        tmp.push(SpanData {
                            span: arg.span,
                            data: AFunctionArg {
                                name: int_name,
                                src: arg.data.name,
                                ty: tmp_ty,
                            },
                        });
                    }
                    tmp
                };

                let declaration = name.0.span.clone();
                parse_state.add_function_declaration(name, declaration, arguments, var_args, r_ty);

                Ok(None)
            }
            Statement::FunctionDefinition {
                head:
                    FunctionHead {
                        name,
                        r_type,
                        arguments,
                        var_args,
                    },
                body,
            } => {
                if parse_state.is_defined(&name) {
                    panic!("Redefinition Error");
                }

                let r_ty = AType::parse(r_type, parse_state.type_defs(), parse_state)?;

                let arguments = {
                    let mut tmp = Vec::new();
                    for arg in arguments {
                        let tmp_ty =
                            AType::parse(arg.data.ty, parse_state.type_defs(), parse_state)?;
                        let name = arg.data.name;

                        let int_name = ParseState::unique_var_name(&name, &arg.span);
                        tmp.push(SpanData {
                            span: arg.span,
                            data: AFunctionArg {
                                name: int_name,
                                src: name,
                                ty: tmp_ty,
                            },
                        });
                    }
                    tmp
                };

                if !parse_state.is_declared(&name) {
                    let declaration = name.0.span.clone();
                    parse_state.add_function_declaration(
                        name.clone(),
                        declaration,
                        arguments.clone(),
                        var_args,
                        r_ty.clone(),
                    );
                }

                let mut function_scope = ParseState::based(parse_state);
                for tmp_arg in arguments.iter() {
                    let arg = &tmp_arg.data;

                    function_scope.add_variable_declaration(
                        arg.src.clone(),
                        tmp_arg.span.clone(),
                        arg.ty.clone(),
                    );
                }
                function_scope.set_return_ty(SpanData {
                    span: name.0.span.clone(),
                    data: r_ty.clone(),
                });

                let inner_scope = AScope::parse(&function_scope, body)?;

                // Check for correct return Statements
                let (expected_r_val_ty, trailing_ret) = match &r_ty {
                    // If the function is of return type Void, there needs to be no return
                    // statement and all return statements that do exist should have no value given
                    AType::Primitve(APrimitive::Void) => (None, false),
                    other => (Some(other), true),
                };

                if trailing_ret {
                    let last = match inner_scope.statements.last() {
                        Some(l) => l,
                        None => {
                            todo!()
                        }
                    };

                    let expected_ty = match expected_r_val_ty {
                        Some(ty) => ty,
                        None => unreachable!("If we expect a trailing Return Statement there also has to be a type set for it"),
                    };

                    let ret_ty = match last {
                        AStatement::Return { value: Some(val) } => val.result_type(),
                        AStatement::Return { value: None } => {
                            todo!()
                        }
                        other => {
                            dbg!(&other);

                            todo!()
                        }
                    };

                    if ret_ty != expected_ty {
                        todo!()
                    }
                }

                let declaration = name.0.span.clone();
                parse_state.add_function_definition(
                    name.0.data,
                    FunctionDeclaration {
                        arguments,
                        declaration,
                        return_ty: r_ty,
                        var_args,
                    },
                    inner_scope,
                );

                Ok(None)
            }
            Statement::VariableDeclaration { ty, name } => {
                let ty = AType::parse(ty, parse_state.type_defs(), parse_state)?;

                if parse_state.is_locally_declared(&name) {
                    panic!("Redeclaration Error");
                }

                let declaration = name.0.span.clone();
                let int_name =
                    parse_state.add_variable_declaration(name.clone(), declaration, ty.clone());

                let result = AStatement::DeclareVar {
                    name: int_name,
                    src: name,
                    ty,
                };

                Ok(Some(result))
            }
            Statement::VariableDeclarationAssignment { ty, name, value } => {
                let ty = AType::parse(ty, parse_state.type_defs(), parse_state)?;

                if parse_state.is_locally_declared(&name) {
                    let prev_dec = parse_state.get_declaration(&name).unwrap();
                    return Err(SemanticError::Redeclaration {
                        name,
                        previous_declaration: prev_dec,
                    });
                }

                let declaration = name.0.span.clone();
                parse_state.add_variable_declaration(name.clone(), declaration, ty);

                // Handle the assign Part of this
                let assign_statement = Statement::VariableAssignment {
                    target: AssignTarget::Variable(name),
                    value,
                };

                AStatement::parse(assign_statement, parse_state)
            }
            Statement::VariableAssignment { target, value } => {
                let base_value_exp =
                    AExpression::parse(value, parse_state.type_defs(), parse_state)?;

                let a_target = AAssignTarget::parse(target, parse_state.type_defs(), parse_state)?;
                let (var_type, var_span) = a_target.get_expected_type();

                let value_exp =
                    atype::assign_type::determine_type(base_value_exp, (&var_type, &var_span))?;

                let exp_type = value_exp.result_type();
                if var_type != exp_type {
                    return Err(SemanticError::MismatchedTypes {
                        expected: SpanData {
                            span: var_span,
                            data: var_type,
                        },
                        received: SpanData {
                            span: value_exp.entire_span(),
                            data: exp_type,
                        },
                    });
                }

                Ok(Some(Self::Assignment {
                    target: a_target,
                    value: value_exp,
                }))
            }
            Statement::VariableDerefAssignment { target, value } => {
                let target_exp = AExpression::parse(target, parse_state.type_defs(), parse_state)?;

                let target_type = target_exp.result_type();

                let inner_ty = match target_type {
                    AType::Pointer(inner) => *inner,
                    other => {
                        dbg!(&other);

                        todo!("Can only dereference a Pointer")
                    }
                };

                let ty_info = SpanData {
                    span: target_exp.entire_span(),
                    data: inner_ty,
                };

                let target = AAssignTarget::Deref {
                    exp: target_exp,
                    ty_info,
                };

                let base_value_exp =
                    AExpression::parse(value, parse_state.type_defs(), parse_state)?;

                let (expected_type, expected_type_span) = target.get_expected_type();
                let value_exp = atype::assign_type::determine_type(
                    base_value_exp,
                    (&expected_type, &expected_type_span),
                )?;

                Ok(Some(Self::Assignment {
                    target,
                    value: value_exp,
                }))
            }
            Statement::SingleExpression(raw_exp) => {
                let exp = AExpression::parse(raw_exp, parse_state.type_defs(), parse_state)?;

                Ok(Some(Self::Expression(exp)))
            }
            Statement::WhileLoop { condition, scope } => {
                let cond = AExpression::parse(condition, parse_state.type_defs(), parse_state)?;

                let inner_scope = AScope::parse(parse_state, scope)?;

                Ok(Some(Self::WhileLoop {
                    condition: cond,
                    body: inner_scope,
                }))
            }
            Statement::ForLoop {
                setup,
                condition,
                update,
                scope,
            } => {
                let mut loop_state = ParseState::based(parse_state);

                let mut a_setups = Vec::new();
                for tmp_setup in setup {
                    if let Some(tmp_setup_a) = AStatement::parse(tmp_setup, &mut loop_state)? {
                        a_setups.push(tmp_setup_a);
                    }
                }

                let mut a_updates = Vec::new();
                for tmp_update in update {
                    if let Some(tmp_update_a) = AStatement::parse(tmp_update, &mut loop_state)? {
                        a_updates.push(tmp_update_a);
                    }
                }

                let a_cond = AExpression::parse(condition, parse_state.type_defs(), &loop_state)?;

                let inner_scope = AScope::parse(&loop_state, scope)?;

                let for_statement = Self::ForLoop {
                    condition: a_cond,
                    updates: a_updates,
                    body: inner_scope,
                };
                let loop_statements: Vec<_> = a_setups
                    .into_iter()
                    .chain(std::iter::once(for_statement))
                    .collect();

                let loop_scope = AScope::from_parse_state(loop_state, loop_statements);

                Ok(Some(Self::SubScope { inner: loop_scope }))
            }
            Statement::Break => Ok(Some(Self::Break)),
            Statement::Continue => Ok(Some(Self::Continue)),
            Statement::If {
                condition,
                scope,
                elses,
            } => {
                let cond = AExpression::parse(condition, parse_state.type_defs(), parse_state)?;

                let inner_scope = AScope::parse(parse_state, scope)?;

                // TODO
                // Parse Elses
                let else_block = match elses {
                    Some(else_inner) => {
                        let else_scope = AScope::parse(parse_state, else_inner)?;
                        Some(else_scope)
                    }
                    None => None,
                };

                Ok(Some(Self::If {
                    condition: cond,
                    body: inner_scope,
                    else_: else_block,
                }))
            }
            Statement::Return(raw_val) => {
                let r_value = match raw_val {
                    Some(raw) => {
                        let value = AExpression::parse(raw, parse_state.type_defs(), parse_state)?;

                        let expected_r_ty = parse_state.return_ty().unwrap();
                        if let AType::Primitve(APrimitive::Void) = expected_r_ty.data {
                            return Err(SemanticError::MismatchedTypes {
                                expected: expected_r_ty.clone(),
                                received: SpanData {
                                    span: value.entire_span(),
                                    data: value.result_type(),
                                },
                            });
                        }

                        let final_val = atype::assign_type::determine_type(
                            value,
                            (&expected_r_ty.data, &expected_r_ty.span),
                        )?;

                        Some(final_val)
                    }
                    None => None,
                };

                Ok(Some(Self::Return { value: r_value }))
            }
            unknown => panic!("Unexpected Statement: {:?}", unknown),
        }
    }

    pub fn to_ir(self, block: &mut BasicBlock, ctx: &ConvertContext) {
        match self {
            Self::SubScope { inner } => {
                let sub_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);
                block.add_statement(ir::Statement::Jump(sub_block.clone()));

                let end_sub_block = inner.to_ir(&sub_block, ctx);
                let following_block = BasicBlock::new(vec![end_sub_block.weak_ptr()], vec![]);
                end_sub_block.add_statement(ir::Statement::Jump(following_block.clone()));

                *block = following_block;
            }
            AStatement::DeclareVar {
                name, ty: raw_ty, ..
            } => {
                let ty = raw_ty.ty();

                let target_name = name;

                match ty {
                    AType::Array(arr) => {
                        let arr_length = arr.size.unwrap();
                        let alignment = arr.ty.alignment(ctx.arch()) as usize;
                        let size = arr_length * arr.ty.byte_size(ctx.arch()) as usize;

                        let ir_ty = arr.ty.to_ir();
                        let target_var =
                            ir::Variable::new(target_name, ir::Type::Pointer(Box::new(ir_ty)))
                                .set_description("Declare Array Variable");

                        let reserve_exp = ir::Expression::StackAlloc { size, alignment };

                        block.add_statement(ir::Statement::Assignment {
                            target: target_var,
                            value: ir::Value::Expression(reserve_exp),
                        });
                    }
                    AType::Struct {
                        def: struct_def, ..
                    } => {
                        let size = struct_def.entire_size(ctx.arch());
                        let alignment = struct_def.alignment(ctx.arch());

                        let ir_ty = ir::Type::Pointer(Box::new(ir::Type::Void));
                        let target_var = ir::Variable::new(target_name, ir_ty)
                            .set_description("Declare Struct Variable");

                        let reserve_exp = ir::Expression::StackAlloc { size, alignment };

                        block.add_statement(ir::Statement::Assignment {
                            target: target_var,
                            value: ir::Value::Expression(reserve_exp),
                        });
                    }
                    AType::Primitve(_) => {
                        let ir_type = ty.to_ir();

                        let var = ir::Variable::new(target_name, ir_type)
                            .set_description("Declare Primitive Variable");
                        block.add_statement(ir::Statement::Assignment {
                            target: var,
                            value: ir::Value::Unknown,
                        });
                    }
                    AType::Pointer(_) => {
                        let ir_type = ty.to_ir();

                        let var = ir::Variable::new(target_name, ir_type)
                            .set_description("Declare Pointer Variable");
                        block.add_statement(ir::Statement::Assignment {
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
                let value_exp = value.to_ir(block, ctx);

                match target {
                    AAssignTarget::Variable { name, ty_info, .. } => {
                        let next_var = match block.definition(&name, &|| ctx.next_tmp()) {
                            Some(var) => var.next_gen(),
                            None => {
                                let target_ty = ty_info.data.to_ir();

                                ir::Variable::new(name.clone(), target_ty)
                            }
                        };

                        let target_meta = value_exp.assign_meta(&next_var);
                        let target_var = next_var.set_meta(target_meta);

                        block.add_statement(ir::Statement::Assignment {
                            target: target_var,
                            value: value_exp,
                        });
                    }
                    AAssignTarget::Deref { exp, .. } => {
                        let address_value = exp.to_ir(block, ctx);

                        let target_oper = AExpression::val_to_operand(address_value, block, ctx);

                        if let ir::Operand::Variable(target_var) = &target_oper {
                            if let ir::VariableMetadata::VarPointer { var } = target_var.meta() {
                                let next_var = var.next_gen();
                                let target_meta = value_exp.assign_meta(&next_var);
                                let target_var = next_var.set_meta(target_meta);

                                block.add_statement(ir::Statement::Assignment {
                                    target: target_var,
                                    value: value_exp.clone(),
                                });
                            }
                        }

                        block.add_statement(ir::Statement::WriteMemory {
                            target: target_oper,
                            value: value_exp,
                        });
                    }
                    AAssignTarget::ArrayAccess(target) => {
                        let target_exp = target.to_exp(block, ctx);

                        let target_value = ir::Value::Expression(target_exp);
                        let target_oper = AExpression::val_to_operand(target_value, block, ctx);

                        block.add_statement(ir::Statement::WriteMemory {
                            target: target_oper,
                            value: value_exp,
                        });
                    }
                    AAssignTarget::StructField(target) => {
                        let target_exp = target.to_exp(block, ctx);

                        let target_value = ir::Value::Expression(target_exp);
                        let target_oper = AExpression::val_to_operand(target_value, block, ctx);

                        block.add_statement(ir::Statement::WriteMemory {
                            target: target_oper,
                            value: value_exp,
                        });
                    }
                };
            }
            AStatement::Expression(exp) => {
                match exp {
                    AExpression::FunctionCall(call) => {
                        call.to_standalone_ir(block, ctx);
                    }
                    AExpression::UnaryOperator { base, op } => {
                        op.to_ir(base, block, ctx);
                    }
                    other => {
                        dbg!(&other);

                        todo!("Unknown Standalone Expression")
                    }
                };
            }
            AStatement::Return { value } => {
                let ret_value = match value {
                    Some(raw_ret) => {
                        let raw_ty = raw_ret.result_type();
                        let target_ty = raw_ty.to_ir();

                        let ret_exp = raw_ret.to_ir(block, ctx);

                        let ret_var = ir::Variable::tmp(ctx.next_tmp(), target_ty);
                        block.add_statement(ir::Statement::Assignment {
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
                let cond_value = condition.to_ir(block, ctx);
                let cond_var = ir::Variable::tmp(ctx.next_tmp(), ir::Type::I64);

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
                end_true_body.add_statement(ir::Statement::Jump(end_block.clone()));
                end_block.add_predecessor(end_true_body.weak_ptr());

                block.add_statement(ir::Statement::JumpTrue(cond_var, true_block));

                if let Some(else_) = else_ {
                    let false_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);
                    let end_false_block = else_.to_ir(&false_block, ctx);
                    end_false_block.add_statement(ir::Statement::Jump(end_block.clone()));
                    block.add_statement(ir::Statement::Jump(end_false_block.clone()));
                    end_block.add_predecessor(end_false_block.weak_ptr());
                } else {
                    // Jump to the end Block directly
                    block.add_statement(ir::Statement::Jump(end_block.clone()));
                    end_block.add_predecessor(block.weak_ptr());
                }

                *block = end_block;
            }
            AStatement::WhileLoop { condition, body } => {
                dbg!(&condition, &body);

                let start_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);
                let inner_block = BasicBlock::new(vec![start_block.weak_ptr()], vec![]);
                let end_block = BasicBlock::new(vec![start_block.weak_ptr()], vec![]);

                for var in condition.used_variables() {
                    let definition: ir::Variable =
                        start_block.definition(&var, &|| ctx.next_tmp()).unwrap();

                    let target = definition.next_gen();

                    start_block.add_statement(ir::Statement::Assignment {
                        target,
                        value: ir::Value::Phi { sources: vec![] },
                    });
                }

                // Generate the first iteration of the start Block
                {
                    let mut cond_start = start_block.clone();
                    let cond_value = condition.to_ir(&mut cond_start, ctx);
                    let cond_var = ir::Variable::tmp(ctx.next_tmp(), ir::Type::I64);

                    let cond_statement = ir::Statement::Assignment {
                        target: cond_var.clone(),
                        value: cond_value,
                    };
                    start_block.add_statement(cond_statement);

                    start_block
                        .add_statement(ir::Statement::JumpTrue(cond_var, inner_block.clone()));
                    start_block.add_statement(ir::Statement::Jump(end_block.clone()));
                }

                // Generate the inner Part of the Loop
                {
                    let loop_ctx = ctx.with_loop(start_block.clone(), end_block.clone());

                    start_block.add_predecessor(inner_block.weak_ptr());

                    let inner_end_block = body.to_ir(&inner_block, &loop_ctx);
                    inner_end_block.add_statement(ir::Statement::Jump(start_block.clone()));
                    start_block.remove_predecessor(inner_block.weak_ptr());
                    start_block.add_predecessor(inner_end_block.weak_ptr());
                }

                start_block.refresh_phis();

                block.add_statement(ir::Statement::Jump(start_block));
                *block = end_block;
            }
            AStatement::ForLoop {
                condition,
                body,
                updates,
            } => {
                dbg!(&condition, &body, &updates);

                let condition_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);

                // The Block containing the Condition
                let cond_var = {
                    for var in condition.used_variables() {
                        let definition: ir::Variable = condition_block
                            .definition(&var, &|| ctx.next_tmp())
                            .unwrap();

                        let target = definition.next_gen();

                        condition_block.add_statement(ir::Statement::Assignment {
                            target,
                            value: ir::Value::Phi { sources: vec![] },
                        });
                    }

                    let mut cond_val_block = condition_block.clone();
                    let cond_value = condition.to_ir(&mut cond_val_block, ctx);
                    let cond_var = ir::Variable::tmp(ctx.next_tmp(), ir::Type::I64);

                    condition_block.add_statement(ir::Statement::Assignment {
                        target: cond_var.clone(),
                        value: cond_value,
                    });

                    cond_var
                };

                // The Block updating some parts after every iteration
                let update_block = BasicBlock::new(vec![], vec![]);
                condition_block.add_predecessor(update_block.weak_ptr());

                // The Block after the Loop
                let end_block = BasicBlock::new(vec![condition_block.weak_ptr()], vec![]);

                // The starting Block of the internal Scope
                let content_start_block = BasicBlock::new(vec![condition_block.weak_ptr()], vec![]);
                {
                    let inner_ctx = ctx.with_loop(update_block.clone(), end_block.clone());

                    let content_end_block = body.to_ir(&content_start_block, &inner_ctx);
                    content_end_block.add_statement(ir::Statement::Jump(update_block.clone()));
                    update_block.add_predecessor(content_end_block.weak_ptr());
                }
                condition_block
                    .add_statement(ir::Statement::JumpTrue(cond_var, content_start_block));
                condition_block.add_statement(ir::Statement::Jump(end_block.clone()));

                // Generate the content of the Update Block
                {
                    let mut end_update_block = update_block.clone();
                    for update in updates {
                        update.to_ir(&mut end_update_block, ctx);

                        assert_eq!(update_block.as_ptr(), end_update_block.as_ptr());
                    }
                }

                block.add_statement(ir::Statement::Jump(condition_block.clone()));

                condition_block.refresh_phis();

                *block = end_block;
            }
            AStatement::Break => {
                let loop_end_block = match ctx.get_loop_end() {
                    Some(b) => b,
                    None => panic!("Break outside of Loop"),
                };

                loop_end_block.add_predecessor(block.weak_ptr());
                block.add_statement(ir::Statement::Jump(loop_end_block.clone()));
            }
            AStatement::Continue => {
                let loop_start_block = match ctx.get_loop_start() {
                    Some(b) => b,
                    None => panic!("Continue outside of loop"),
                };

                loop_start_block.add_predecessor(block.weak_ptr());
                block.add_statement(ir::Statement::Jump(loop_start_block.clone()));
            }
        };
    }
}
