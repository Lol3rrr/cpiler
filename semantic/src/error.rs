use general::{Span, SpanData};
use syntax::Identifier;

use crate::{AType, StructDef};

#[derive(Debug, PartialEq, Clone)]
pub enum SemanticError {
    MismatchedTypes {
        expected: SpanData<AType>,
        received: SpanData<AType>,
    },
    AmbiguousTypeConversion {
        target: SpanData<AType>,
        base: SpanData<AType>,
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
    UnknownType {
        name: Identifier,
    },
    UnknownStructField {
        /// The Name of the Field that the Code tried to access
        field_name: Identifier,
        /// The Definition of the Struct itself
        struct_def: SpanData<StructDef>,
    },
    StructAccessOnNonStruct {
        field_name: Identifier,
        received: SpanData<AType>,
    },
    MismatchedFunctionArgsCount {
        expected: SpanData<usize>,
        received: SpanData<usize>,
    },
}
