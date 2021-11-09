use tokenizer::TokenData;

use crate::directive;

#[derive(Debug, PartialEq, Clone)]
pub enum PIR {
    Token(tokenizer::Token),
    Directive((tokenizer::Token, directive::Directive)),
}

pub fn into_pir<I, IT>(iter: I) -> impl Iterator<Item = PIR>
where
    I: IntoIterator<IntoIter = IT, Item = tokenizer::Token>,
    IT: Iterator<Item = tokenizer::Token>,
{
    iter.into_iter().map(|t| match &t.data {
        TokenData::CompilerDirective { content } => {
            let dir = directive::Directive::parse((&t.span).into()).unwrap();
            PIR::Directive((t, dir))
        }
        _ => PIR::Token(t),
    })
}
