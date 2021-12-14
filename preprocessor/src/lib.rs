use std::{path::PathBuf, str::FromStr};

use tokenizer::{tokenize, Token};

mod directive;
pub mod loader;
pub use loader::Loader;

mod resolver;

mod pir;
use pir::{into_pir, PIR};

mod state;

#[derive(Debug)]
pub enum ProcessError<L> {
    UnknownDirective { raw: String },
    Loading(L),
}

pub fn preprocess<L>(loader: &L, start: &str) -> Result<Vec<Token>, ProcessError<L::LoadError>>
where
    L: Loader,
{
    let start_path = PathBuf::from_str(start).unwrap();
    let start_load_directive = loader::LoadDirective {
        local_root: Some(PathBuf::from_str("").unwrap()),
        relative_path: start_path,
    };

    let root = match loader.load_file(start_load_directive) {
        Ok(r) => r,
        Err(e) => return Err(ProcessError::Loading(e)),
    };

    let root_tokens = tokenize(root);
    let root_pir = into_pir(root_tokens);

    let mut state = state::State::new();
    let processed = resolver::resolve(root_pir, loader, &mut state);

    let result = processed
        .map(|p| match p {
            PIR::Token(t) => t,
            PIR::Directive(d) => panic!("Unresolved Compiler Directive: {:?}", d),
        })
        .collect();

    Ok(result)
}
