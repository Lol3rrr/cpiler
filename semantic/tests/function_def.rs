use std::collections::HashMap;

use general::{Source, Span};
use semantic::{ARootScope, AScope, AAST};

#[test]
fn function_def_args() {
    let input = "
void test(int arg_1, int arg_2) {
    int rand = arg_1 + arg_2;
}
        ";

    let input_source = Source::new("test", input);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(input_tokens).unwrap();

    let expected = Ok(AAST {
        global_scope: ARootScope(AScope {
            statements: vec![],
            function_definitions: HashMap::new(),
        }),
    });

    let result = semantic::parse(input_ast);

    assert_eq!(expected, result);
}
