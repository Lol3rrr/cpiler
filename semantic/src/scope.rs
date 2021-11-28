use std::collections::HashMap;

use syntax::Scope;

use crate::{AStatement, FunctionDeclaration, SemanticError};

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
}
