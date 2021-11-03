use std::{collections::HashMap, iter::Peekable};

use tokenizer::{Token, TokenData};

use crate::{pir::PIR, steps::define::function};

use super::RegisteredDefine;

pub fn get_defined<'d>(
    token: &Token,
    defines: &'d HashMap<String, RegisteredDefine>,
) -> Option<&'d RegisteredDefine> {
    let literal = match &token.data {
        TokenData::Literal { content } => content,
        _ => return None,
    };

    defines.get(literal)
}

pub fn expand<I>(
    tok_iter: &mut Peekable<I>,
    defined: &RegisteredDefine,
    macros: &HashMap<String, RegisteredDefine>,
) -> Option<Vec<Token>>
where
    I: Iterator<Item = PIR>,
{
    match defined {
        RegisteredDefine::Block { content } => Some(content.clone()),
        RegisteredDefine::Function { arguments, content } => {
            match tok_iter.peek() {
                Some(PIR::Token(tok)) if matches!(&tok.data, TokenData::OpenParen) => {}
                _ => return None,
            };

            let called_args = match function::parse_call_args(tok_iter) {
                Some(a) => a,
                None => panic!("Expected Args"),
            };

            dbg!(&called_args);

            if called_args.len() != arguments.len() {
                panic!(
                    "Expected {:?} Arguments but got {:?}",
                    arguments.len(),
                    called_args.len()
                );
            }

            let arg_map: HashMap<_, _> = arguments
                .iter()
                .map(|a| a.to_owned())
                .zip(called_args.into_iter())
                .collect();

            dbg!(&arg_map);

            let expanded = function::expand_function_macro(macros, arg_map, &content);

            dbg!(&expanded);

            Some(expanded)
        }
    }
}
