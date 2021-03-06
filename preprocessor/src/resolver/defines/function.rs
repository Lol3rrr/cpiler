use std::{
    collections::HashMap,
    ops::{Deref, Range},
    sync::Arc,
};

use general::{Source, Span, SpanData};
use tokenizer::{Token, TokenData};

use super::DefineManager;
use crate::PIR;

use super::expand;

pub fn parse_call_args<I>(iter: &mut I) -> Option<Vec<Vec<Token>>>
where
    I: Iterator<Item = PIR>,
{
    match iter.next() {
        Some(PIR::Token(t)) => match t.data {
            TokenData::OpenParen => {}
            _ => return None,
        },
        _ => return None,
    };

    let mut result = Vec::new();

    let mut current_param = Vec::new();
    loop {
        let current = match iter.next() {
            Some(c) => c,
            None => return None,
        };

        let inner_token = match current {
            PIR::Token(t) => match &t.data {
                TokenData::CloseParen => break,
                _ => t,
            },
            PIR::Directive(_) => {
                panic!("Unexpected Directive");
            }
        };

        if inner_token.data == TokenData::Comma {
            result.push(current_param);
            current_param = Vec::new();
            continue;
        }

        current_param.push(inner_token);
    }

    result.push(current_param);

    Some(result)
}

#[derive(Debug)]
pub enum MacroToken {
    Original(Token),
    Param(Token),
    Created(Token),
}

impl Deref for MacroToken {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Original(t) => t,
            Self::Param(t) => t,
            Self::Created(t) => t,
        }
    }
}
impl From<MacroToken> for Token {
    fn from(src: MacroToken) -> Self {
        match src {
            MacroToken::Original(t) => t,
            MacroToken::Param(t) => t,
            MacroToken::Created(t) => t,
        }
    }
}

// Order of expansion:
// - Stringification operations are replaced with the textual representation of their argument's replacement list (without performing expansion).
// - Parameters are replaced with their replacement list (without performing expansion).
// - Concatenation operations are replaced with the concatenated result of the two operands (without expanding the resulting token).
// - Tokens originating from parameters are expanded.
// - The resulting tokens are expanded as normal.
pub fn expand_function_macro(
    og: (&Arc<Source>, &Range<usize>),
    macros: &DefineManager,
    call_args: HashMap<String, Vec<Token>>,
    macro_content: &[Token],
) -> Vec<Token> {
    // TODO
    // Stringification
    let stringified = stringify(macro_content.iter().map(Token::clone));

    // Replaced Parameters with their arguments
    let replaced_params = replace_params(stringified, &call_args);

    let concat_idents = concat_idents(replaced_params);

    // Expand token Parameters
    let param_expanded = expand_params(og, concat_idents, macros);

    // Expand all the other Tokens
    expand_all(og, param_expanded, macros)
}

fn stringify<I, IT>(input: I) -> Vec<MacroToken>
where
    I: IntoIterator<Item = Token, IntoIter = IT>,
    IT: Iterator<Item = Token>,
{
    let mut result = Vec::new();

    let prev_iter = input.into_iter();
    for current in prev_iter {
        result.push(MacroToken::Original(current));
    }

    result
}

fn replace_params<I, IT>(input: I, arguments: &HashMap<String, Vec<Token>>) -> Vec<MacroToken>
where
    I: IntoIterator<Item = MacroToken, IntoIter = IT>,
    IT: Iterator<Item = MacroToken>,
{
    let mut result = Vec::new();

    for m_tok in input.into_iter() {
        match &m_tok.data {
            TokenData::Literal { content } if arguments.contains_key(content) => {
                let replacement = arguments.get(content).unwrap();
                result.extend(replacement.iter().map(|t| MacroToken::Param(t.clone())));
            }
            _ => {
                result.push(m_tok);
            }
        };
    }

    result
}

fn concat_idents<I, IT>(input: I) -> Vec<MacroToken>
where
    I: IntoIterator<Item = MacroToken, IntoIter = IT>,
    IT: Iterator<Item = MacroToken>,
{
    let mut result = Vec::new();

    let mut prev_iter = input.into_iter().peekable();
    while let Some(current) = prev_iter.next() {
        match prev_iter.peek() {
            Some(f_peeked) => {
                match &f_peeked.data {
                    TokenData::Hashtag => {}
                    _ => {
                        result.push(current);
                        continue;
                    }
                };
            }
            None => {
                result.push(current);
                continue;
            }
        };

        let f_hashtag = prev_iter.next().unwrap();

        match prev_iter.peek() {
            Some(s_peeked) => {
                match &s_peeked.data {
                    TokenData::Hashtag => {}
                    _ => {
                        result.push(f_hashtag);
                        continue;
                    }
                };
            }
            _ => {
                result.push(f_hashtag);
                continue;
            }
        };

        let s_hashtag = prev_iter.next().unwrap();

        let second_part = match prev_iter.next() {
            Some(t) => t,
            None => {
                result.push(f_hashtag);
                result.push(s_hashtag);
                continue;
            }
        };

        let raw_new = {
            let mut tmp = current.data.to_string();
            tmp.push_str(&second_part.data.to_string());
            tmp
        };

        let source = Source::new("preprocessor", raw_new);
        let new = tokenizer::tokenize(source.into());

        result.extend(new.into_iter().map(MacroToken::Created));
    }

    result
}

fn expand_params<I, IT>(
    og: (&Arc<Source>, &Range<usize>),
    input: I,
    macros: &DefineManager,
) -> Vec<Token>
where
    I: IntoIterator<Item = MacroToken, IntoIter = IT>,
    IT: Iterator<Item = MacroToken>,
{
    let mut result = Vec::new();

    let mut prev_iter = input.into_iter();
    while let Some(tmp) = prev_iter.next() {
        match tmp {
            MacroToken::Param(t) => {
                match &t.data {
                    TokenData::Literal { content } if macros.is_defined(content) => {
                        let macro_def = macros
                            .get_defined(content)
                            .expect("We just checked that a macro for this name exists");

                        let mut tmp_iter = prev_iter
                            .by_ref()
                            .map(|t| PIR::Token((*t).clone()))
                            .peekable();

                        match expand(
                            (t.span.source(), t.span.source_area()),
                            &mut tmp_iter,
                            macro_def,
                            macros,
                        ) {
                            Some(resulting) => {
                                result.extend(resulting);
                            }
                            None => {
                                result.push(t);
                            }
                        };
                    }
                    _ => {
                        result.push(t);
                    }
                };
            }
            t => {
                let inner: SpanData<TokenData> = t.into();
                dbg!(&inner);

                let n_inner = SpanData {
                    span: Span::new_arc_source_og(og.0.clone(), og.1.clone(), inner.span.clone()),
                    data: inner.data.clone(),
                };
                dbg!(&n_inner);

                result.push(n_inner);
            }
        };
    }

    result
}

fn expand_all<I, IT>(
    og: (&Arc<Source>, &Range<usize>),
    input: I,
    macros: &DefineManager,
) -> Vec<Token>
where
    I: IntoIterator<Item = Token, IntoIter = IT>,
    IT: Iterator<Item = Token>,
{
    let mut result = Vec::new();

    let mut prev_iter = input.into_iter();
    while let Some(current) = prev_iter.next() {
        match &current.data {
            TokenData::Literal { content } if macros.is_defined(content) => {
                let macro_def = macros
                    .get_defined(content)
                    .expect("We just checked that a Macro for this Name exists");

                let mut tmp_iter = prev_iter.by_ref().map(PIR::Token).peekable();

                match expand(
                    (current.span.source(), current.span.source_area()),
                    &mut tmp_iter,
                    macro_def,
                    macros,
                ) {
                    Some(resulting) => {
                        result.extend(resulting.into_iter().map(|t| SpanData {
                            span: Span::new_arc_source_og(og.0.clone(), og.1.clone(), t.span),
                            data: t.data,
                        }));
                    }
                    None => {
                        result.push(current);
                    }
                };
            }
            _ => {
                result.push(current);
            }
        };
    }

    result
}
