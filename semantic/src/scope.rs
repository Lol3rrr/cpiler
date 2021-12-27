use std::collections::HashMap;

use ir::BasicBlock;
use syntax::Scope;

use crate::{conversion::ConvertContext, AStatement, FunctionDeclaration, SemanticError};

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

#[derive(Debug, PartialEq, Clone)]
pub struct AScope {
    pub statements: Vec<AStatement>,
    pub function_definitions: HashMap<String, (FunctionDeclaration, AScope)>,
}

impl AScope {
    pub fn parse(external_state: &ParseState, scope: Scope) -> Result<Self, SemanticError> {
        let mut current = ParseState::based(external_state);

        let mut statements: Vec<AStatement> = Vec::new();

        for statement in scope.statements {
            if let Some(tmp) = AStatement::parse(statement, &mut current)? {
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
            tmp_stmnt.to_ir(&mut block, ctx);
        }

        block
    }
}
