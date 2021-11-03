use general::Span;
use tokenizer::{tokenize, Token};

mod directive;
pub mod loader;

mod steps;

mod pir;
use pir::{into_pir, PIR};

pub trait Loader {
    type LoadError: std::error::Error;

    /// Loads the File at the given Path relative to the current Directory/Root
    fn load_file(&self, path: &str) -> Result<Span, Self::LoadError>;

    fn load_as_pir(&self, path: &str) -> Result<Vec<PIR>, Self::LoadError> {
        let span = self.load_file(path)?;

        let tokens = tokenizer::tokenize(span);

        let pir = into_pir(tokens);
        Ok(pir.collect())
    }
}

#[derive(Debug)]
pub enum ProcessError<L> {
    UnknownDirective { raw: String },
    Loading(L),
}

pub fn preprocess<L>(loader: &L, start: &str) -> Result<Vec<Token>, ProcessError<L::LoadError>>
where
    L: Loader,
{
    let root = match loader.load_file(start) {
        Ok(r) => r,
        Err(e) => return Err(ProcessError::Loading(e)),
    };

    let root_tokens = tokenize(root);
    let root_pir = into_pir(root_tokens);

    let included = steps::handle_include(loader, root_pir).unwrap();

    let defined = steps::handle_define(included).unwrap();

    let result = defined
        .into_iter()
        .map(|p| match p {
            PIR::Token(t) => t,
            PIR::Directive(d) => panic!("Unresolved Compiler Directive: {:?}", d),
        })
        .collect();

    return Ok(result);
}
