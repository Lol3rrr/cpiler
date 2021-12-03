use std::collections::HashMap;

use general::{Span, SpanData};
use syntax::Identifier;

use crate::{
    AFunctionArg, AScope, AType, FunctionDeclaration, TypeDefinitions, VariableContainer,
    VariableDeclaration,
};

#[derive(Debug)]
pub struct ParseState {
    external_variables: Variables,
    local_variables: Variables,
    type_defs: TypeDefinitions,
    function_definitions: HashMap<String, (FunctionDeclaration, AScope)>,
}

impl ParseState {
    pub fn new() -> Self {
        Self {
            external_variables: Variables::new(),
            local_variables: Variables::new(),
            type_defs: TypeDefinitions::new(),
            function_definitions: HashMap::new(),
        }
    }

    pub fn based(other: &Self) -> Self {
        let new_ext = other.local_variables.join(&other.external_variables);
        let type_defs = other.type_defs.clone();

        Self {
            external_variables: new_ext,
            local_variables: Variables::new(),
            type_defs,
            function_definitions: HashMap::new(),
        }
    }

    pub fn type_defs(&self) -> &TypeDefinitions {
        &self.type_defs
    }
    pub fn mut_type_defs(&mut self) -> &mut TypeDefinitions {
        &mut self.type_defs
    }

    pub fn is_declared(&self, ident: &Identifier) -> bool {
        if self.local_variables.is_declared(ident) {
            return true;
        }

        self.external_variables.is_declared(ident)
    }
    pub fn get_declaration(&self, ident: &Identifier) -> Option<Span> {
        if let Some(dec) = self.local_variables.get_declared(ident) {
            match dec {
                Declared::Variable(var) => return Some(var.declaration.clone()),
                Declared::Function(func) => return Some(func.declaration.clone()),
            };
        }

        if let Some(dec) = self.external_variables.get_declared(ident) {
            match dec {
                Declared::Variable(var) => return Some(var.declaration.clone()),
                Declared::Function(func) => return Some(func.declaration.clone()),
            };
        }

        None
    }

    pub fn is_defined(&self, ident: &Identifier) -> bool {
        if self.local_variables.is_defined(ident) {
            return true;
        }

        self.external_variables.is_defined(ident)
    }

    pub fn add_function_declaration(
        &mut self,
        name: Identifier,
        declaration: Span,
        arguments: Vec<SpanData<AFunctionArg>>,
        var_args: bool,
        return_ty: AType,
    ) {
        self.local_variables.declare_function(
            name,
            FunctionDeclaration {
                return_ty,
                arguments,
                declaration,
                var_args,
            },
        );
    }

    pub fn add_variable_declaration(&mut self, name: Identifier, declaration: Span, ty: AType) {
        dbg!(&name);
        self.local_variables.declare_variable(name, ty, declaration);
    }

    pub fn add_function_definition(
        &mut self,
        name: String,
        func_dec: FunctionDeclaration,
        scope: AScope,
    ) {
        self.function_definitions.insert(name, (func_dec, scope));
    }

    pub fn destructure(self) -> HashMap<String, (FunctionDeclaration, AScope)> {
        self.function_definitions
    }
}

impl VariableContainer for ParseState {
    fn get_var(&self, ident: &Identifier) -> Option<(&AType, &Span)> {
        match self.local_variables.get_declared(ident) {
            Some(Declared::Variable(var)) => return Some((&var.ty, &var.declaration)),
            _ => {}
        };

        self.external_variables
            .get_declared(ident)
            .into_iter()
            .find_map(|d| match d {
                Declared::Variable(var) => Some((&var.ty, &var.declaration)),
                _ => None,
            })
    }

    fn get_func(&self, ident: &Identifier) -> Option<&FunctionDeclaration> {
        match self.local_variables.get_declared(ident) {
            Some(Declared::Function(func)) => {
                return Some(func);
            }
            _ => {}
        };

        self.external_variables
            .get_declared(ident)
            .into_iter()
            .find_map(|d| match d {
                Declared::Function(func) => Some(func),
                _ => None,
            })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Declared {
    Variable(VariableDeclaration),
    Function(FunctionDeclaration),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Defined {
    Function,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variables(HashMap<String, Declared>, HashMap<String, Defined>);

impl Variables {
    pub fn new() -> Self {
        Self(HashMap::new(), HashMap::new())
    }

    pub fn is_declared(&self, ident: &Identifier) -> bool {
        self.0.contains_key(&ident.0.data)
    }
    pub fn is_defined(&self, ident: &Identifier) -> bool {
        self.1.contains_key(&ident.0.data)
    }

    pub fn declare_variable(&mut self, name: Identifier, ty: AType, declaration: Span) {
        let data = Declared::Variable(VariableDeclaration { ty, declaration });
        self.0.insert(name.0.data, data);
    }
    pub fn declare_function(&mut self, name: Identifier, func_dec: FunctionDeclaration) {
        let data = Declared::Function(func_dec);

        self.0.insert(name.0.data, data);
    }

    pub fn get_declared(&self, ident: &Identifier) -> Option<&Declared> {
        let name = &ident.0.data;
        self.0.get(name)
    }

    fn join(&self, other: &Self) -> Self {
        let mut declared = self.0.clone();
        let mut defined = self.1.clone();

        for (ident, dec) in other.0.clone() {
            declared.insert(ident, dec);
        }
        for (ident, def) in other.1.clone() {
            defined.insert(ident, def);
        }

        Self(declared, defined)
    }
}
