use std::{iter::Peekable, path::PathBuf, str::FromStr};

use crate::{directive::Directive, loader::LoadDirective, pir::PIR, Loader};

mod defines;
pub use defines::DefineManager;
use tokenizer::TokenData;

mod conditionals;

pub fn resolve<I, L>(pir: I, loader: &L, defines: &mut DefineManager) -> std::vec::IntoIter<PIR>
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
                    TokenData::Literal { content } if defines.is_defined(&content) => {
                        let m_def = defines
                            .get_defined(&content)
                            .expect("We previously checked that this Key is defined");

                        match defines::expand(&mut new_iter, &m_def, &defines) {
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
                        if local {
                            let mut local_root = PathBuf::from_str(span.span.source()).unwrap();
                            local_root.pop();

                            let load_directive = LoadDirective {
                                local_root: Some(local_root),
                                relative_path: PathBuf::from_str(&path).unwrap(),
                            };
                            let raw_included = loader.load_as_pir(load_directive).unwrap();

                            let full = resolve(raw_included.into_iter(), loader, defines);

                            result.extend(full);
                        } else {
                            todo!("Include non local file");
                        }
                    }
                    Directive::DefineBlock { name, body } => {
                        let tokenized = tokenizer::tokenize(body);

                        defines.add_block(name, tokenized);
                    }
                    Directive::DefineFunction {
                        name,
                        arguments,
                        body,
                    } => {
                        let tokenized = tokenizer::tokenize(body);

                        defines.add_function(name, arguments, tokenized);
                    }
                    Directive::Undefine { name } => {
                        defines.remove_defined(&name);
                    }
                    Directive::If { condition } => {
                        let condition = conditionals::parse_conditional(condition).unwrap();

                        let cond = condition.evaluate(&defines).unwrap();
                        let tmp = evaluate_conditional(&mut new_iter, loader, defines, cond);
                        result.extend(tmp);
                    }
                    Directive::IfDef { name } => {
                        let cond = defines.is_defined(&name);
                        let tmp = evaluate_conditional(&mut new_iter, loader, defines, cond);
                        result.extend(tmp);
                    }
                    Directive::IfNDef { name } => {
                        let cond = !defines.is_defined(&name);
                        let tmp = evaluate_conditional(&mut new_iter, loader, defines, cond);
                        result.extend(tmp);
                    }
                    other => {
                        dbg!(other);
                    }
                };
            }
        };
    }
    dbg!(&result);

    result.into_iter()
}

fn evaluate_conditional<I, L>(
    iter: &mut Peekable<I>,
    loader: &L,
    defines: &mut DefineManager,
    cond: bool,
) -> Vec<PIR>
where
    I: Iterator<Item = PIR>,
    L: Loader,
{
    if cond {
        let inner_iter = conditionals::InnerConditionalIterator::new(iter);

        resolve(inner_iter, loader, defines).collect()
    } else {
        while let Some(peeked) = iter.peek() {
            let directive = match &peeked {
                PIR::Directive((_, dir)) => dir,
                _ => {
                    iter.next();
                    continue;
                }
            };

            match directive {
                Directive::Endif => break,
                Directive::Else => break,
                _ => {}
            };
        }

        Vec::new()
    }
}
