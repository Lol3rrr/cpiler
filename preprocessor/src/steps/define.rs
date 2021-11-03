use std::collections::HashMap;

use crate::{directive::Directive, PIR};

mod expand;
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

pub fn handle_define<PII, PIT>(pir: PII) -> Result<Vec<PIR>, ()>
where
    PII: IntoIterator<Item = PIR, IntoIter = PIT>,
    PIT: Iterator<Item = PIR>,
{
    let mut defines: HashMap<String, RegisteredDefine> = HashMap::new();

    let mut result = Vec::new();

    let mut pir_iter = pir.into_iter().peekable();
    while let Some(pir_token) = pir_iter.next() {
        match pir_token {
            PIR::Token(t) => {
                let m_def = match expand::get_defined(&t, &defines) {
                    Some(m) => m,
                    None => {
                        result.push(PIR::Token(t));
                        continue;
                    }
                };

                match expand::expand(&mut pir_iter, &m_def, &defines) {
                    Some(expanded) => {
                        result.extend(expanded.into_iter().map(|t| PIR::Token(t)));
                    }
                    None => {
                        result.push(PIR::Token(t));
                    }
                };
            }
            PIR::Directive((t, d)) => {
                match d {
                    Directive::DefineBlock { name, body } => {
                        let tokenized = tokenizer::tokenize(body);

                        defines.insert(name, RegisteredDefine::Block { content: tokenized });
                    }
                    Directive::DefineFunction {
                        name,
                        arguments,
                        body,
                    } => {
                        let tokenized = tokenizer::tokenize(body);

                        defines.insert(
                            name,
                            RegisteredDefine::Function {
                                arguments,
                                content: tokenized,
                            },
                        );
                    }
                    Directive::Undefine { name } => {
                        defines.remove(&name);
                    }
                    d => {
                        result.push(PIR::Directive((t, d)));
                    }
                };
            }
        };
    }

    Ok(result)
}
