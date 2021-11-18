use preprocessor::Loader;

#[derive(Debug)]
pub enum Error<P> {
    Preprocessor(preprocessor::ProcessError<P>),
    Syntax(syntax::SyntaxError),
}

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
