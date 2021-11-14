use preprocessor::Loader;

pub fn run<L>(source_file: &str, loader: L)
where
    L: Loader,
{
    let preprocessed = preprocessor::preprocess(&loader, source_file).unwrap();

    let basic_ast = syntax::parse(preprocessed);

    dbg!(basic_ast);
}
