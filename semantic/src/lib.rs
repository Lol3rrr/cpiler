#![allow(clippy::large_enum_variant)]

use std::collections::HashMap;

use general::{Span, SpanData};
use ir::Program;
use syntax::{Identifier, AST};

mod scope;
pub use scope::*;

mod atype;
pub use atype::*;

mod aexpression;
pub use aexpression::*;

mod astatement;
pub use astatement::*;

mod type_definitions;
pub use type_definitions::TypeDefinitions;

mod error;
pub use error::*;

mod conversion;

#[derive(Debug, PartialEq, Clone)]
pub struct VariableDeclaration {
    internal_name: String,
    ty: AType,
    declaration: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AFunctionArg {
    pub name: String,
    pub src: Identifier,
    pub ty: AType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration {
    pub return_ty: AType,
    pub declaration: Span,
    pub arguments: Vec<SpanData<AFunctionArg>>,
    pub var_args: bool,
}

/// An Annotated Abstract Syntax Tree
#[derive(Debug, PartialEq)]
pub struct AAST {
    pub global_scope: ARootScope,
}

pub fn parse(ast: AST) -> Result<AAST, SemanticError> {
    let global_scope = ARootScope::parse(ast.global_scope)?;

    Ok(AAST { global_scope })
}

pub trait VariableContainer {
    fn get_var(&self, ident: &Identifier) -> Option<&VariableDeclaration>;

    fn get_func(&self, ident: &Identifier) -> Option<&FunctionDeclaration>;
}

pub enum FuncOrVar {
    Var(VariableDeclaration),
    Function(FunctionDeclaration),
}
impl VariableContainer for HashMap<String, FuncOrVar> {
    fn get_var(&self, ident: &Identifier) -> Option<&VariableDeclaration> {
        let ident_name = &ident.0.data;
        self.get(ident_name).into_iter().find_map(|r| match r {
            FuncOrVar::Var(v) => Some(v),
            _ => None,
        })
    }

    fn get_func(&self, ident: &Identifier) -> Option<&FunctionDeclaration> {
        let ident_name = &ident.0.data;
        self.get(ident_name).into_iter().find_map(|r| match r {
            FuncOrVar::Function(f) => Some(f),
            _ => None,
        })
    }
}

impl AAST {
    pub fn convert_to_ir(self, arch: general::arch::Arch) -> Program {
        conversion::convert(self, arch)
    }
}
