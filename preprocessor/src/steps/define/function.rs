use std::{collections::HashMap, ops::Deref};

use general::Span;
use tokenizer::{Token, TokenData};

use super::RegisteredDefine;
use crate::{steps::define::expand, PIR};

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

        match &inner_token.data {
            TokenData::Comma => {
                result.push(current_param);
                current_param = Vec::new();
                continue;
            }
            _ => {}
        };

        current_param.push(inner_token);
    }

    result.push(current_param);

    Some(result)
}

#[derive(Debug)]
pub enum MacroToken {
    Original(Token),
    Replaced(Token),
    Created(Token),
}

impl Deref for MacroToken {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Original(t) => t,
            Self::Replaced(t) => t,
            Self::Created(t) => t,
        }
    }
}
impl Into<Token> for MacroToken {
    fn into(self) -> Token {
        match self {
            Self::Original(t) => t,
            Self::Replaced(t) => t,
            Self::Created(t) => t,
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
    macros: &HashMap<String, RegisteredDefine>,
    call_args: HashMap<String, Vec<Token>>,
    macro_content: &[Token],
) -> Vec<Token> {
    // TODO
    // Stringification
    let stringified = stringify(macro_content.iter().map(|t| Token::clone(t)));

    // Replaced Parameters with their arguments
    let replaced_params = replace_params(stringified, &call_args);

    let concat_idents = concat_idents(replaced_params);

    // Expand token Parameters
    let param_expanded = expand_params(concat_idents, macros);

    // Expand all the other Tokens
    let all_expanded = expand_all(param_expanded, macros);

    let result = all_expanded;

    result
}

fn stringify<I, IT>(input: I) -> Vec<MacroToken>
where
    I: IntoIterator<Item = Token, IntoIter = IT>,
    IT: Iterator<Item = Token>,
{
    let mut result = Vec::new();

    let mut prev_iter = input.into_iter();
    while let Some(current) = prev_iter.next() {
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
                result.extend(
                    replacement
                        .into_iter()
                        .map(|t| MacroToken::Replaced(t.clone())),
                );
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

        let new = tokenizer::tokenize(Span::from_parts("preprocessor", &raw_new, 0..raw_new.len()));

        result.extend(new.into_iter().map(|t| MacroToken::Created(t)));
    }

    result
}

fn expand_params<I, IT>(input: I, macros: &HashMap<String, RegisteredDefine>) -> Vec<Token>
where
    I: IntoIterator<Item = MacroToken, IntoIter = IT>,
    IT: Iterator<Item = MacroToken>,
{
    let mut result = Vec::new();

    let mut prev_iter = input.into_iter();
    while let Some(tmp) = prev_iter.next() {
        match tmp {
            MacroToken::Replaced(t) => {
                if let Some(macro_def) = expand::get_defined(&t, macros) {
                    let mut tmp_iter = prev_iter
                        .by_ref()
                        .map(|t| PIR::Token((*t).clone()))
                        .peekable();

                    match expand::expand(&mut tmp_iter, macro_def, macros) {
                        Some(resulting) => {
                            result.extend(resulting);
                        }
                        None => {
                            result.push(t);
                        }
                    };
                } else {
                    result.push(t);
                }
            }
            t => {
                result.push((*t).clone());
            }
        };
    }

    result
}

fn expand_all<I, IT>(input: I, macros: &HashMap<String, RegisteredDefine>) -> Vec<Token>
where
    I: IntoIterator<Item = Token, IntoIter = IT>,
    IT: Iterator<Item = Token>,
{
    let mut result = Vec::new();

    let mut prev_iter = input.into_iter();
    while let Some(current) = prev_iter.next() {
        if let Some(macro_def) = expand::get_defined(&current, &macros) {
            let mut tmp_iter = prev_iter.by_ref().map(|t| PIR::Token(t)).peekable();

            match expand::expand(&mut tmp_iter, macro_def, macros) {
                Some(resulting) => {
                    result.extend(resulting);
                }
                None => {
                    result.push(current);
                }
            };
        } else {
            result.push(current);
        }
    }

    result
}
