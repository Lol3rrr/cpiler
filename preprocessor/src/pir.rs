use tokenizer::TokenData;

use crate::directive;

#[derive(Debug, PartialEq, Clone)]
pub enum PIR {
    Token(tokenizer::Token),
    Directive((tokenizer::Token, directive::Directive)),
}

pub struct PirIterator<I> {
    token_iter: I,
}

pub fn into_pir<I, IT>(iter: I) -> PirIterator<IT>
where
    I: IntoIterator<IntoIter = IT, Item = tokenizer::Token>,
    IT: Iterator<Item = tokenizer::Token>,
{
    PirIterator {
        token_iter: iter.into_iter(),
    }
}

impl<I> Iterator for PirIterator<I>
where
    I: Iterator<Item = tokenizer::Token>,
{
    type Item = PIR;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_iter.next().map(|t| match &t.data {
            TokenData::CompilerDirective { .. } => {
                let dir = directive::Directive::parse((&t.span).into()).unwrap();
                PIR::Directive((t, dir))
            }
            _ => PIR::Token(t),
        })
    }
}
