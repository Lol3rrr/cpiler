#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: general::Span| {
    let mut tokens = tokenizer::tokenize(data);

    let ast = match syntax::parse(tokens.by_ref()) {
        Ok(a) => a,
        Err(_) => return,
    };

    //fuzzed code goes here
    let parsed = semantic::parse(ast);
});
