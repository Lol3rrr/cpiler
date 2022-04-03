use std::{cell::RefCell, path::PathBuf, rc::Rc, str::FromStr, sync::Arc};

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
    UnknownDirective {
        raw: String,
    },
    FailedInclude {
        directive: general::Span,
        path: String,
        error: L,
    },
    Loading(L),
}

pub fn preprocess<L>(loader: Arc<L>, start: &str) -> Result<Vec<Token>, ProcessError<L::LoadError>>
where
    L: Loader + 'static,
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
    state.defines.add_block("CPILER", Vec::new());

    let processed = resolver::resolve(root_pir, loader, Rc::new(RefCell::new(state)));

    let result: Result<Vec<_>, _> = processed
        .map(|rp| {
            rp.map(|p| match p {
                PIR::Token(t) => t,
                PIR::Directive(d) => panic!("Unresolved Compiler Directive: {:?}", d),
            })
        })
        .collect();

    result
}
