use std::collections::HashMap;

use general::Span;
use syntax::Identifier;

use crate::{AType, Declared, FunctionDeclaration, VariableDeclaration};

/// The Local State for a Scope
#[derive(Debug)]
pub struct LocalState {
    vars: HashMap<String, VariableDeclaration>,
    funcs: HashMap<String, FunctionDeclaration>,
}

impl LocalState {
    /// Creates a new empty State for a new Scope
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            funcs: HashMap::new(),
        }
    }

    /// Checks if a Variable is declared in this Scope
    pub fn is_declared(&self, ident: &Identifier) -> bool {
        self.vars.contains_key(&ident.0.data)
    }

    /// Gets the Declaration Span for a given Variable
    pub fn get_declaration(&self, ident: &Identifier) -> Option<Span> {
        if let Some(var_dec) = self.vars.get(&ident.0.data) {
            return Some(var_dec.declaration.clone());
        }

        if let Some(func_dec) = self.funcs.get(&ident.0.data) {
            return Some(func_dec.declaration.clone());
        }

        None
    }

    /// Gets the Declaration for the given Identifier
    pub fn get_var_declared(&self, ident: &Identifier) -> Option<&VariableDeclaration> {
        self.vars.get(&ident.0.data)
    }
    pub fn get_func_defined(&self, ident: &Identifier) -> Option<&FunctionDeclaration> {
        self.funcs.get(&ident.0.data)
    }

    /// Checks if there is a definition for a given Ident
    pub fn is_defined(&self, ident: &Identifier) -> bool {
        self.funcs.contains_key(&ident.0.data)
    }

    /// Adds a Variable Declaration for this Scope
    pub fn declare_var(&mut self, name: Identifier, ty: AType, decl: Span) {
        self.vars.insert(
            name.0.data,
            VariableDeclaration {
                ty,
                declaration: decl,
            },
        );
    }
    /// Adds a Function Declaration for this Scope
    pub fn declare_func(&mut self, name: Identifier, func: FunctionDeclaration) {
        self.funcs.insert(name.0.data, func);
    }
}
