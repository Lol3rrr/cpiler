use general::{Span, SpanData};
use syntax::Identifier;

use crate::AType;

#[derive(Debug, PartialEq, Clone)]
pub enum SemanticError {
    MismatchedTypes {
        expected: SpanData<AType>,
        received: SpanData<AType>,
    },
    Redefinition {
        name: Identifier,
        previous_definition: Span,
    },
    UnknownIdentifier {
        name: Identifier,
    },
    MismatchedFunctionArgsCount {
        expected: SpanData<usize>,
        received: SpanData<usize>,
    },
}
