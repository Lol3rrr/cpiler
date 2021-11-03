use std::{path::PathBuf, str::FromStr};

use crate::{directive::Directive, Loader, PIR};

pub fn handle_include<L, PII, PIT>(loader: &L, pir: PII) -> Result<Vec<PIR>, L::LoadError>
where
    L: Loader,
    PII: IntoIterator<Item = PIR, IntoIter = PIT>,
    PIT: Iterator<Item = PIR>,
{
    let iter = pir.into_iter();

    let mut result = Vec::new();
    for pir_token in iter {
        match pir_token {
            PIR::Token(t) => {
                result.push(PIR::Token(t));
            }
            PIR::Directive((t, d)) => {
                match d {
                    Directive::Include { path, local } => {
                        if local {
                            let mut source_path = PathBuf::from_str(t.span.source()).unwrap();
                            source_path.pop();
                            source_path.push(&path);
                            let final_path = source_path;

                            let raw_included = loader.load_as_pir(final_path.to_str().unwrap())?;
                            let included = handle_include(loader, raw_included)?;

                            result.extend(included);
                        } else {
                            todo!("Include non local file");
                        }
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
