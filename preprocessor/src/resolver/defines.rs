use std::{collections::HashMap, iter::Peekable, ops::Range, sync::Arc};

use general::{Source, Span, SpanData};
use tokenizer::TokenData;

use crate::pir::PIR;

mod function;

#[derive(Debug, PartialEq)]
pub enum RegisteredDefine {
    Block {
        content: Vec<tokenizer::Token>,
    },
    Function {
        arguments: Vec<String>,
        content: Vec<tokenizer::Token>,
    },
}

#[derive(Debug)]
pub struct DefineManager {
    inner: HashMap<String, RegisteredDefine>,
}

impl DefineManager {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.inner.contains_key(name)
    }

    pub fn get_defined<'s>(&'s self, name: &str) -> Option<&'s RegisteredDefine> {
        self.inner.get(name)
    }

    pub fn remove_defined(&mut self, name: &str) {
        self.inner.remove(name);
    }

    pub fn add_block<N>(&mut self, name: N, content: Vec<tokenizer::Token>)
    where
        N: Into<String>,
    {
        let defined = RegisteredDefine::Block { content };

        self.inner.insert(name.into(), defined);
    }

    pub fn add_function(
        &mut self,
        name: String,
        arguments: Vec<String>,
        content: Vec<tokenizer::Token>,
    ) {
        let defined = RegisteredDefine::Function { arguments, content };

        self.inner.insert(name, defined);
    }
}

impl Default for DefineManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn expand<I>(
    tok: (&Arc<Source>, &Range<usize>),
    tok_iter: &mut Peekable<I>,
    defined: &RegisteredDefine,
    macros: &DefineManager,
) -> Option<Vec<tokenizer::Token>>
where
    I: Iterator<Item = PIR>,
{
    match defined {
        RegisteredDefine::Block { content } => {
            let iter = content.clone().into_iter().map(|t| {
                let n_span = Span::new_arc_source_og(tok.0.clone(), tok.1.clone(), t.span);

                SpanData {
                    span: n_span,
                    data: t.data,
                }
            });

            Some(iter.collect())
        }
        RegisteredDefine::Function { arguments, content } => {
            match tok_iter.peek() {
                Some(PIR::Token(tok)) if matches!(&tok.data, TokenData::OpenParen) => {}
                _ => return None,
            };

            let called_args = match function::parse_call_args(tok_iter) {
                Some(a) => a,
                None => panic!("Expected Args"),
            };

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

            let expanded = function::expand_function_macro(tok, macros, arg_map, content);

            Some(expanded)
        }
    }
}
