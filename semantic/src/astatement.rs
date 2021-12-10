use general::SpanData;
use syntax::{AssignTarget, FunctionHead, Statement};

use crate::{
    atype, AExpression, AFunctionArg, AScope, AType, FunctionDeclaration, ParseState, SemanticError,
};

mod target;
pub use target::*;

#[derive(Debug, PartialEq)]
pub enum AStatement {
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

                dbg!(&name, &target_ty);

                parse_state.mut_type_defs().add_definition(name, target_ty);

                Ok(None)
            }
            Statement::StructDefinition { name, members } => {
                dbg!(&name, &members);

                let ty = AType::parse_struct(members, parse_state.type_defs(), parse_state)?;
                dbg!(&ty);

                parse_state.mut_type_defs().add_definition(name, ty);
                Ok(None)
            }
            Statement::FunctionDeclaration(FunctionHead {
                name,
                r_type,
                arguments,
                var_args,
            }) => {
                dbg!(&name, &r_type, &arguments);

                if parse_state.is_declared(&name) {
                    panic!("Redefinition Error");
                }

                let r_ty = AType::parse(r_type, parse_state.type_defs(), parse_state)?;

                let arguments = {
                    let mut tmp = Vec::new();
                    for arg in arguments {
                        let tmp_ty =
                            AType::parse(arg.data.ty, parse_state.type_defs(), parse_state)?;
                        tmp.push(SpanData {
                            span: arg.span,
                            data: AFunctionArg {
                                name: arg.data.name,
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
                dbg!(&name, &r_type, &arguments, &var_args, &body);

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

                        tmp.push(SpanData {
                            span: arg.span,
                            data: AFunctionArg { name, ty: tmp_ty },
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

                let mut function_scope = ParseState::based(&parse_state);
                for tmp_arg in arguments.iter() {
                    let arg = &tmp_arg.data;

                    function_scope.add_variable_declaration(
                        arg.name.clone(),
                        tmp_arg.span.clone(),
                        arg.ty.clone(),
                    );
                }

                let inner_scope = AScope::parse(&function_scope, body)?;

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

                dbg!(&name, &ty);

                if parse_state.is_declared(&name) {
                    panic!("Redefintion Error");
                }

                let declaration = name.0.span.clone();
                parse_state.add_variable_declaration(name, declaration, ty);

                Ok(None)
            }
            Statement::VariableDeclarationAssignment { ty, name, value } => {
                let ty = AType::parse(ty, parse_state.type_defs(), parse_state)?;

                dbg!(&name, &ty);

                if parse_state.is_declared(&name) {
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

                dbg!(&value_exp);

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
            Statement::SingleExpression(raw_exp) => {
                let exp = AExpression::parse(raw_exp, parse_state.type_defs(), parse_state)?;

                Ok(Some(Self::Expression(exp)))
            }
            Statement::WhileLoop { condition, scope } => {
                dbg!(&condition, &scope);

                let cond = AExpression::parse(condition, parse_state.type_defs(), parse_state)?;
                dbg!(&cond);

                let inner_scope = AScope::parse(parse_state, scope)?;
                dbg!(&inner_scope);

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
                dbg!(&setup, &condition, &update, &scope);

                let mut loop_state = ParseState::based(&parse_state);

                let mut a_setups = Vec::new();
                for tmp_setup in setup {
                    if let Some(tmp_setup_a) = AStatement::parse(tmp_setup, &mut loop_state)? {
                        dbg!(&tmp_setup_a);

                        a_setups.push(tmp_setup_a);
                    }
                }

                let mut a_updates = Vec::new();
                for tmp_update in update {
                    if let Some(tmp_update_a) = AStatement::parse(tmp_update, &mut loop_state)? {
                        dbg!(&tmp_update_a);

                        a_updates.push(tmp_update_a);
                    }
                }

                let a_cond = AExpression::parse(condition, parse_state.type_defs(), &loop_state)?;

                dbg!(&a_setups, &a_cond, &a_updates);

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
            Statement::If {
                condition,
                scope,
                elses,
            } => {
                dbg!(&condition, &scope, &elses);

                let cond = AExpression::parse(condition, parse_state.type_defs(), parse_state)?;
                dbg!(&cond);

                let inner_scope = AScope::parse(parse_state, scope)?;
                dbg!(&inner_scope);

                // TODO
                // Parse Elses
                let else_block = match elses {
                    Some(else_inner) => {
                        dbg!(&else_inner);

                        let else_scope = AScope::parse(&parse_state, else_inner)?;
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
                dbg!(&raw_val);
                let r_value = match raw_val {
                    Some(raw) => {
                        let value = AExpression::parse(raw, parse_state.type_defs(), parse_state)?;

                        Some(value)
                    }
                    None => None,
                };

                Ok(Some(Self::Return { value: r_value }))
            }
            unknown => panic!("Unexpected Statement: {:?}", unknown),
        }
    }
}
