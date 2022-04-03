use std::{cell::RefCell, iter::Peekable, path::PathBuf, rc::Rc, str::FromStr, sync::Arc};

use crate::{
    directive::Directive, loader::LoadDirective, pir::PIR, state::State, Loader, ProcessError,
};

mod defines;
pub use defines::DefineManager;
use tokenizer::TokenData;

mod conditionals;

mod extensions;

pub fn resolve<'s, I, L>(
    pir: I,
    loader: Arc<L>,
    state: Rc<RefCell<State>>,
) -> impl Iterator<Item = Result<PIR, ProcessError<L::LoadError>>> + 's
where
    I: Iterator<Item = PIR> + 's,
    L: Loader + 'static,
{
    return ResolveIterator {
        loader,
        state,
        tmp: Box::new(std::iter::empty()),
        pir_iter: pir.into_iter().peekable(),
    };
}

pub struct ResolveIterator<I, L>
where
    L: Loader,
    L::LoadError: 'static,
    I: Iterator<Item = PIR>,
{
    pir_iter: Peekable<I>,
    loader: Arc<L>,
    tmp: Box<dyn Iterator<Item = Result<PIR, ProcessError<L::LoadError>>>>,
    state: Rc<RefCell<State>>,
}

impl<'s, I, L> Iterator for ResolveIterator<I, L>
where
    L: Loader + 'static,
    L::LoadError: 'static,
    I: Iterator<Item = PIR>,
{
    type Item = Result<PIR, ProcessError<L::LoadError>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tmp) = self.tmp.next() {
            return Some(tmp);
        }

        loop {
            let current = self.pir_iter.next()?;
            let mut state = self.state.borrow_mut();

            match current {
                PIR::Token(tok) => {
                    match &tok.data {
                        TokenData::Literal { content } if state.defines.is_defined(content) => {
                            let m_def = state
                                .defines
                                .get_defined(content)
                                .expect("We previously checked that this Key is defined");

                            match defines::expand(
                                (tok.span.source(), tok.span.source_area()),
                                &mut self.pir_iter,
                                m_def,
                                &state.defines,
                            ) {
                                Some(replacements) => {
                                    self.tmp = Box::new(
                                        replacements.into_iter().map(|t| Ok(PIR::Token(t))),
                                    );

                                    drop(state);
                                    return self.tmp.next();
                                }
                                None => return Some(Ok(PIR::Token(tok))),
                            };
                        }
                        _ => return Some(Ok(PIR::Token(tok))),
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
                            let raw_included = match self
                                .loader
                                .load_as_pir(load_directive, &mut state)
                                .map_err(|e| ProcessError::FailedInclude {
                                    directive: span.span.clone(),
                                    path: path.to_string(),
                                    error: e,
                                }) {
                                Ok(r) => r,
                                Err(e) => return Some(Err(e)),
                            };

                            self.tmp = Box::new(resolve(
                                raw_included,
                                self.loader.clone(),
                                self.state.clone(),
                            ));

                            drop(state);

                            return self.tmp.next();
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
                            drop(state);
                            let tmp = match evaluate_conditional(
                                &mut self.pir_iter,
                                &self.loader,
                                self.state.clone(),
                                condition,
                            ) {
                                Ok(e) => e,
                                Err(e) => return Some(Err(e)),
                            };
                            self.tmp = Box::new(tmp.into_iter().map(Ok));
                            return self.tmp.next();
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
    }
}

fn evaluate_conditional<I, L>(
    iter: &mut Peekable<I>,
    loader: &Arc<L>,
    rstate: Rc<RefCell<State>>,
    cond: conditionals::Conditional,
) -> Result<Vec<PIR>, ProcessError<L::LoadError>>
where
    I: Iterator<Item = PIR>,
    L: Loader + 'static,
{
    let state = rstate.borrow_mut();
    if cond.evaluate(&state.defines).unwrap() {
        let inner_iter = conditionals::InnerConditionalIterator::new(iter);

        drop(state);
        let pir_iter = resolve(inner_iter, Arc::clone(loader), rstate);
        pir_iter.collect()
    } else {
        let mut load_inner = false;
        for peeked in iter.by_ref() {
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

            drop(state);
            let inner_res = resolve(inner_iter, Arc::clone(loader), rstate);
            inner_res.collect()
        } else {
            Ok(Vec::new())
        }
    }
}
