use preprocessor::Loader;

mod error;
pub use error::Error;

pub fn run<L>(source_file: &str, loader: L) -> Result<(), Error<L::LoadError>>
where
    L: Loader,
{
    let preprocessed =
        preprocessor::preprocess(&loader, source_file).map_err(|e| Error::Preprocessor(e))?;

    let basic_ast = syntax::parse(preprocessed).map_err(|e| Error::Syntax(e))?;

    dbg!(basic_ast);

    Ok(())
}
