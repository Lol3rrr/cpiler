use general::{Span, SpanData};
use syntax::Identifier;

use crate::{AType, StructDef};

#[derive(Debug, PartialEq, Clone)]
pub enum SemanticError {
    MismatchedTypes {
        expected: SpanData<AType>,
        received: SpanData<AType>,
    },
    InvalidType {},
    Redeclaration {
        name: Identifier,
        previous_declaration: Span,
    },
    Redefinition {
        name: Identifier,
        previous_definition: Span,
    },
    UnknownIdentifier {
        name: Identifier,
    },
    UnknownStructField {
        /// The Name of the Field that the Code tried to access
        field_name: Identifier,
        /// The Definition of the Struct itself
        struct_def: SpanData<StructDef>,
    },
    MismatchedFunctionArgsCount {
        expected: SpanData<usize>,
        received: SpanData<usize>,
    },
}
