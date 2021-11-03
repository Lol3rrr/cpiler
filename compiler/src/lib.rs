use preprocessor::loader::files::FileLoader;

pub fn run(source_file: &str) {
    let loader = FileLoader::new();

    let preprocessed = preprocessor::preprocess(&loader, source_file).unwrap();

    let basic_ast = syntax::parse(preprocessed);

    dbg!(basic_ast);
}
