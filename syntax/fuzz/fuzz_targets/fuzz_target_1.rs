#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    // fuzzed code goes here
    let source = general::Source::new("test", data);
    let span: general::Span = source.clone().into();
    let mut tokens = tokenizer::tokenize(span);

    let result = syntax::parse(tokens.by_ref());
});
