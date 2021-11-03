use std::iter::Peekable;

use tokenizer::{Token, TokenData};

use crate::Statement;

#[derive(Debug, PartialEq)]
pub struct Scope {
    pub statements: Vec<Statement>,
}

impl Scope {
    pub fn parse<I>(tokens: &mut Peekable<I>) -> Self
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

            let statement = Statement::parse(tokens).unwrap();

            statements.push(statement);
        }

        Self { statements }
    }
}
