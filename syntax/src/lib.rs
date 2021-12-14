use itertools::peek_nth;
use tokenizer::{Token, TokenData};

mod scope;
pub use scope::Scope;

mod statement;
pub use statement::{
    AssignTarget, EnumVariant, EnumVariants, FunctionHead, Statement, StructMembers, TypeDefType,
};

mod expression;
pub use expression::{Expression, ExpressionOperator, SingleOperation};

mod identifier;
pub use identifier::Identifier;

mod ty;
pub use ty::{Modifier, TypeToken};

mod error;
pub use error::*;

pub use tokenizer::DataType;

#[derive(Debug, PartialEq)]
pub struct FunctionArgument {
    /// The Name of the Argument
    pub name: Identifier,
    /// The Type of the Argument
    pub ty: TypeToken,
}

#[derive(Debug, PartialEq)]
pub struct AST {
    pub global_scope: Scope,
}

pub fn parse<I, IT>(tokens: I) -> Result<AST, SyntaxError>
where
    I: IntoIterator<Item = Token, IntoIter = IT>,
    IT: Iterator<Item = Token>,
{
    let mut tokens = peek_nth(
        tokens
            .into_iter()
            .filter(|t| !matches!(&t.data, TokenData::Comment { .. })),
    );

    let global_scope = Scope::parse(&mut tokens)?;

    Ok(AST { global_scope })
}
