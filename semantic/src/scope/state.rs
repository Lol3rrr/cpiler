use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use general::{Span, SpanData};
use syntax::Identifier;

use crate::{
    AFunctionArg, AScope, AType, FunctionDeclaration, TypeDefinitions, VariableContainer,
    VariableDeclaration,
};

mod local;
use local::LocalState;

#[derive(Debug)]
pub struct ParseState<'p> {
    parent: Option<&'p Self>,
    local: LocalState,
    type_defs: TypeDefinitions,
    function_definitions: HashMap<String, (FunctionDeclaration, AScope)>,
    return_ty: Option<SpanData<AType>>,
}

impl<'p> ParseState<'p> {
    pub fn new() -> Self {
        Self {
            parent: None,
            local: LocalState::new(),
            type_defs: TypeDefinitions::new(),
            function_definitions: HashMap::new(),
            return_ty: None,
        }
    }

    pub fn based(other: &'p ParseState<'_>) -> Self {
        let type_defs = other.type_defs.clone();

        Self {
            parent: Some(other),
            local: LocalState::new(),
            type_defs,
            function_definitions: HashMap::new(),
            return_ty: other.return_ty.clone(),
        }
    }

    pub fn type_defs(&self) -> &TypeDefinitions {
        &self.type_defs
    }
    pub fn mut_type_defs(&mut self) -> &mut TypeDefinitions {
        &mut self.type_defs
    }

    pub fn set_return_ty(&mut self, ty: SpanData<AType>) {
        self.return_ty = Some(ty);
    }
    pub fn return_ty(&self) -> Option<&SpanData<AType>> {
        self.return_ty.as_ref()
    }

    pub fn is_declared(&self, ident: &Identifier) -> bool {
        if self.local.is_declared(ident) {
            return true;
        }

        match self.parent {
            Some(p) => p.is_declared(ident),
            _ => false,
        }
    }
    pub fn is_locally_declared(&self, ident: &Identifier) -> bool {
        self.local.is_declared(ident)
    }
    pub fn get_declaration(&self, ident: &Identifier) -> Option<Span> {
        if let Some(dec) = self.local.get_declaration(ident) {
            return Some(dec);
        }

        match self.parent {
            Some(p) => p.get_declaration(ident),
            None => None,
        }
    }

    pub fn is_defined(&self, ident: &Identifier) -> bool {
        if self.local.is_defined(ident) {
            return true;
        }

        match self.parent {
            Some(p) => p.is_defined(ident),
            None => false,
        }
    }

    pub fn add_function_declaration(
        &mut self,
        name: Identifier,
        declaration: Span,
        arguments: Vec<SpanData<AFunctionArg>>,
        var_args: bool,
        return_ty: AType,
    ) {
        self.local.declare_func(
            name,
            FunctionDeclaration {
                return_ty,
                arguments,
                declaration,
                var_args,
            },
        );
    }

    pub fn add_variable_declaration(
        &mut self,
        name: Identifier,
        declaration: Span,
        ty: AType,
    ) -> String {
        self.local.declare_var(name, ty, declaration)
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

impl Default for ParseState<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseState<'_> {
    pub fn unique_var_name(name: &Identifier, decl: &Span) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.0.data.hash(&mut hasher);
        decl.content().hash(&mut hasher);
        decl.source().name().hash(&mut hasher);
        hasher.write_usize(decl.source_area().start);
        hasher.write_usize(decl.source_area().end);
        let id = hasher.finish();
        format!("{}_{}", name.0.data, id)
    }
}

impl VariableContainer for ParseState<'_> {
    fn get_var(&self, ident: &Identifier) -> Option<&VariableDeclaration> {
        if let Some(var) = self.local.get_var_declared(ident) {
            return Some(var);
        }

        match self.parent {
            Some(p) => p.get_var(ident),
            None => None,
        }
    }

    fn get_func(&self, ident: &Identifier) -> Option<&FunctionDeclaration> {
        if let Some(func) = self.local.get_func_defined(ident) {
            return Some(func);
        }

        match self.parent {
            Some(p) => p.get_func(ident),
            None => None,
        }
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

    pub fn declare_variable(&mut self, name: Identifier, ty: AType, declaration: Span) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.0.data.hash(&mut hasher);
        declaration.content().hash(&mut hasher);
        declaration.source().name().hash(&mut hasher);
        hasher.write_usize(declaration.source_area().start);
        hasher.write_usize(declaration.source_area().end);
        let id = hasher.finish();
        let internal_name = format!("{}_{}", name.0.data, id);

        let data = Declared::Variable(VariableDeclaration {
            internal_name: internal_name.clone(),
            ty,
            declaration,
        });
        self.0.insert(name.0.data, data);

        internal_name
    }
    pub fn declare_function(&mut self, name: Identifier, func_dec: FunctionDeclaration) {
        let data = Declared::Function(func_dec);

        self.0.insert(name.0.data, data);
    }

    pub fn get_declared(&self, ident: &Identifier) -> Option<&Declared> {
        let name = &ident.0.data;
        self.0.get(name)
    }
}

impl Default for Variables {
    fn default() -> Self {
        Self::new()
    }
}
