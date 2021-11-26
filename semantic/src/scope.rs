use std::collections::HashMap;

use syntax::{AssignTarget, Scope, Statement};

use crate::{AStatement, AType, FunctionDeclaration, SemanticError};

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

        let mut statements = Vec::new();
        let mut function_definitions = HashMap::new();

        for statement in scope.statements {
            match statement {
                Statement::TypeDef { name, base_type } => {
                    let target_ty = AType::parse_typedef(base_type, current.type_defs(), &current)?;

                    dbg!(&name, &target_ty);

                    current.mut_type_defs().add_definition(name, target_ty);
                }
                Statement::VariableDeclaration { ty, name } => {
                    let ty = AType::parse(ty, current.type_defs(), &current)?;

                    dbg!(&name, &ty);

                    if current.is_declared(&name) {
                        panic!("Redefintion Error");
                    }

                    let declaration = name.0.span.clone();
                    current.add_variable_declaration(name, declaration, ty);
                }
                Statement::VariableDeclarationAssignment { ty, name, value } => {
                    let ty = AType::parse(ty, current.type_defs(), &current)?;

                    dbg!(&name, &ty);

                    if current.is_declared(&name) {
                        panic!("Redefintion Error");
                    }

                    let declaration = name.0.span.clone();
                    current.add_variable_declaration(name.clone(), declaration, ty);

                    // Handle the assign Part of this
                    let assign_statement = Statement::VariableAssignment {
                        target: AssignTarget::Variable(name),
                        value,
                    };
                    let tmp_stmnt = AStatement::parse(
                        assign_statement,
                        &current,
                        &current,
                        current.type_defs(),
                    )?;
                    dbg!(&tmp_stmnt);

                    statements.push(tmp_stmnt);
                }
                Statement::FunctionDeclaration {
                    name,
                    r_type,
                    arguments,
                } => {
                    dbg!(&name, &r_type, &arguments);

                    if current.is_declared(&name) {
                        panic!("Redefinition Error");
                    }

                    let r_ty = AType::parse(r_type, current.type_defs(), &current)?;

                    let arguments = {
                        let mut tmp = Vec::new();
                        for arg in arguments {
                            let tmp_ty = AType::parse(arg.ty, current.type_defs(), &current)?;
                            tmp.push(tmp_ty);
                        }
                        tmp
                    };

                    let declaration = name.0.span.clone();
                    current.add_function_declaration(name, declaration, arguments, r_ty);
                }
                Statement::FunctionDefinition {
                    name,
                    r_type,
                    arguments,
                    body,
                } => {
                    dbg!(&name, &r_type, &arguments, &body);

                    if current.is_defined(&name) {
                        panic!("Redefinition Error");
                    }

                    let r_ty = AType::parse(r_type, current.type_defs(), &current)?;

                    let arguments = {
                        let mut tmp = Vec::new();
                        for arg in arguments {
                            let tmp_ty = AType::parse(arg.ty, current.type_defs(), &current)?;
                            tmp.push(tmp_ty);
                        }
                        tmp
                    };

                    if !current.is_declared(&name) {
                        let declaration = name.0.span.clone();
                        current.add_function_declaration(
                            name.clone(),
                            declaration,
                            arguments.clone(),
                            r_ty.clone(),
                        );
                    }

                    let inner_scope = AScope::parse(&current, body)?;

                    let declaration = name.0.span.clone();
                    function_definitions.insert(
                        name.0.data,
                        (
                            FunctionDeclaration {
                                arguments,
                                declaration,
                                return_ty: r_ty,
                            },
                            inner_scope,
                        ),
                    );
                }
                unknown => {
                    let tmp_stmnt =
                        AStatement::parse(unknown, &current, &current, current.type_defs())?;
                    dbg!(&tmp_stmnt);

                    statements.push(tmp_stmnt);
                }
            };
        }

        Ok(Self {
            statements,
            function_definitions,
        })
    }
}
