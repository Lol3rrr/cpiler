use general::{Span, SpanData};

mod state;

mod tokendata;
pub use tokendata::*;

mod iter;
pub use iter::TokenIter;

pub type Token = SpanData<TokenData>;

pub fn tokenize(content: Span) -> TokenIter {
    iter::TokenIter::new(content)
}
