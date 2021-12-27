use syntax::Identifier;

use crate::{AExpression, AType};

#[derive(Debug, PartialEq, Clone)]
pub struct StructAccess {
    pub base: Box<AExpression>,
    pub field: Identifier,
    pub ty: AType,
}
