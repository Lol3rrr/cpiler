use std::{iter::Peekable, path::PathBuf, str::FromStr};

use crate::{directive::Directive, loader::LoadDirective, pir::PIR, state::State, Loader};

mod defines;
pub use defines::DefineManager;
use tokenizer::TokenData;

mod conditionals;

mod extensions;

pub fn resolve<I, L>(pir: I, loader: &L, state: &mut State) -> std::vec::IntoIter<PIR>
where
    I: Iterator<Item = PIR>,
    L: Loader,
{
    let mut result = Vec::new();
    let mut new_iter = pir.into_iter().peekable();
    while let Some(current) = new_iter.next() {
        match current {
            PIR::Token(tok) => {
                match &tok.data {
                    TokenData::Literal { content } if state.defines.is_defined(&content) => {
                        let m_def = state
                            .defines
                            .get_defined(&content)
                            .expect("We previously checked that this Key is defined");

                        match defines::expand(
                            (tok.span.source(), tok.span.source_area()),
                            &mut new_iter,
                            &m_def,
                            &state.defines,
                        ) {
                            Some(replacements) => {
                                result.extend(replacements.into_iter().map(|t| PIR::Token(t)));
                            }
                            None => {
                                result.push(PIR::Token(tok));
                            }
                        };
                    }
                    _ => {
                        result.push(PIR::Token(tok));
                    }
                };
            }
            PIR::Directive((span, dir)) => {
                match dir {
                    Directive::Include { path, local } => {
                        let local_root = if local {
                            let mut local_root =
                                PathBuf::from_str(span.span.source().name()).unwrap();
                            local_root.pop();
                            Some(local_root)
                        } else {
                            None
                        };

                        let load_directive = LoadDirective {
                            local_root,
                            relative_path: PathBuf::from_str(&path).unwrap(),
                        };
                        let raw_included = loader.load_as_pir(load_directive, state).unwrap();

                        let full = resolve(raw_included.into_iter(), loader, state);

                        result.extend(full);
                    }
                    Directive::DefineBlock { name, body } => {
                        let tokenized = tokenizer::tokenize(body).collect();

                        state.defines.add_block(name, tokenized);
                    }
                    Directive::DefineFunction {
                        name,
                        arguments,
                        body,
                    } => {
                        let tokenized = tokenizer::tokenize(body).collect();

                        state.defines.add_function(name, arguments, tokenized);
                    }
                    Directive::Undefine { name } => {
                        state.defines.remove_defined(&name);
                    }
                    Directive::Conditional(cond) => {
                        let condition: conditionals::Conditional = cond.try_into().unwrap();
                        let tmp = evaluate_conditional(&mut new_iter, loader, state, condition);
                        result.extend(tmp);
                    }
                    Directive::Pragma { content } => {
                        let span_content = content.content();

                        if span_content.starts_with("GCC") {
                            extensions::gcc::pragma(content);
                        } else {
                            dbg!(&content);
                            todo!()
                        }
                    }
                    Directive::Extensions(extensions) => {
                        dbg!(&extensions);
                        todo!("Extensions are currently not support")
                    }
                    other => {
                        todo!("Unexpcted: {:?}", other);
                    }
                };
            }
        };
    }

    result.into_iter()
}

fn evaluate_conditional<I, L>(
    iter: &mut Peekable<I>,
    loader: &L,
    state: &mut State,
    cond: conditionals::Conditional,
) -> Vec<PIR>
where
    I: Iterator<Item = PIR>,
    L: Loader,
{
    dbg!(&cond);
    if cond.evaluate(&state.defines).unwrap() {
        let inner_iter = conditionals::InnerConditionalIterator::new(iter);

        resolve(inner_iter, loader, state).collect()
    } else {
        let mut load_inner = false;
        while let Some(peeked) = iter.next() {
            let directive = match peeked {
                PIR::Directive((_, dir)) => dir,
                _ => {
                    continue;
                }
            };

            match directive {
                Directive::EndIf => break,
                Directive::Conditional(cond) => {
                    let condition: conditionals::Conditional = cond.try_into().unwrap();

                    if condition.evaluate(&state.defines).unwrap() {
                        load_inner = true;
                        break;
                    }
                }
                _ => {}
            };
        }

        if load_inner {
            let inner_iter = conditionals::InnerConditionalIterator::new(iter);

            resolve(inner_iter, loader, state).collect()
        } else {
            Vec::new()
        }
    }
}
