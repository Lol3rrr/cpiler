#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: general::Span| {
    // fuzzed code goes here
    let mut tokens = tokenizer::tokenize(data);

    let result = syntax::parse(tokens.by_ref());
});
