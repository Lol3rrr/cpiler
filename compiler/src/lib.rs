use preprocessor::Loader;

mod error;
pub use error::Error;

pub fn run<L>(source_file: &str, loader: L) -> Result<(), Error<L::LoadError>>
where
    L: Loader,
{
    let preprocessed =
        preprocessor::preprocess(&loader, source_file).map_err(Error::Preprocessor)?;

    let basic_ast = syntax::parse(preprocessed).map_err(Error::Syntax)?;

    dbg!(&basic_ast);

    let aast = semantic::parse(basic_ast).map_err(Error::Semantic)?;

    dbg!(&aast);

    let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

    std::fs::write("./program.dot", ir.to_dot()).expect("");

    Ok(())
}
