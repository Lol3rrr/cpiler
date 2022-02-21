#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    if let Ok(s) = std::str::from_utf8(data) {
        let source = general::Source::new("test", s);
        let span: general::Span = source.clone().into();
        let mut tokens = tokenizer::tokenize(span);

        let result = syntax::parse(tokens.by_ref());
    }
});
