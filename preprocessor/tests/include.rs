use std::sync::Arc;

use general::Source;
use preprocessor::loader::files::FileLoader;

#[test]
fn simple_include() {
    let loader = FileLoader::new();

    let other_source = Source::new("./tests/files/other.c", include_str!("./files/other.c"));
    let include_source = Source::new("./tests/files/include.c", include_str!("./files/include.c"));

    let other_tokens = tokenizer::tokenize(other_source.into());
    let include_tokens = {
        let mut tmp = tokenizer::tokenize(include_source.into());
        tmp.next();
        tmp
    };

    let expected: Vec<_> = other_tokens.chain(include_tokens).collect();

    let result = preprocessor::preprocess(Arc::new(loader), "./tests/files/include.c").unwrap();

    assert_eq!(expected, result);
}
