use itertools::PeekNth;
use tokenizer::{ControlFlow, Keyword, Token, TokenData};

use crate::{ExpectedToken, Scope, Statement, SyntaxError};

pub fn parse<I>(tokens: &mut PeekNth<I>) -> Result<Scope, SyntaxError>
where
    I: Iterator<Item = Token>,
{
    let peeked_tok = tokens.peek().ok_or(SyntaxError::UnexpectedEOF)?;
    match &peeked_tok.data {
        TokenData::OpenBrace => {
            let _ = tokens.next();

            Scope::parse(tokens)
        }
        TokenData::Keyword(Keyword::ControlFlow(ControlFlow::If)) => {
            let if_statement = Statement::parse(tokens, &Statement::default_terminaton())?;
            dbg!(&if_statement);

            Ok(Scope {
                statements: vec![if_statement],
            })
        }
        _ => {
            let next = tokens.next().unwrap();

            Err(SyntaxError::UnexpectedToken {
                got: next.span,
                expected: Some(vec![ExpectedToken::If, ExpectedToken::OpenBrace]),
            })
        }
    }
}
