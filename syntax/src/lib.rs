use general::{Span, SpanData};
use tokenizer::{DataType, Token};

mod scope;
pub use scope::Scope;

mod statement;
pub use statement::Statement;

mod expression;
pub use expression::{Expression, ExpressionOperator, SingleOperation};

mod identifier;
pub use identifier::Identifier;

mod ty;
pub use ty::TypeToken;

#[derive(Debug, PartialEq)]
pub enum SyntaxError {
    UnexpectedToken {
        expected: Option<Vec<String>>,
        got: Span,
    },
    UnexpectedEOF,
}

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

pub fn parse<I, IT>(tokens: I) -> AST
where
    I: IntoIterator<Item = Token, IntoIter = IT>,
    IT: Iterator<Item = Token>,
{
    let mut tokens = tokens.into_iter().peekable();

    let global_scope = Scope::parse(&mut tokens);

    AST { global_scope }
}
