use itertools::PeekNth;
use tokenizer::{Token, TokenData};

use crate::{Statement, SyntaxError};

#[derive(Debug, PartialEq)]
pub struct Scope {
    pub statements: Vec<Statement>,
}

impl Scope {
    /// Assumes that the Opening Curly Brace is not in the Iterator anymore.
    /// Will consume the Closing Curly Brace
    pub fn parse<I>(tokens: &mut PeekNth<I>) -> Result<Self, SyntaxError>
    where
        I: Iterator<Item = Token>,
    {
        let mut statements = Vec::new();

        while let Some(peeked) = tokens.peek() {
            match &peeked.data {
                TokenData::CloseBrace => {
                    tokens.next();
                    break;
                }
                _ => {}
            };

            let statement = Statement::parse(tokens, &Statement::default_terminaton())?;

            dbg!(&statement);

            statements.push(statement);
        }

        Ok(Self { statements })
    }
}
