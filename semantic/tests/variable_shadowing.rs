use general::{Source, Span};

#[test]
fn shadow_in_condition() {
    let content = "
int main() {
    int x = 0;

    if (1) {
        int x = 13;
    }

    return 0;
}
        ";
    let input_source = Source::new("test", content);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(input_tokens).unwrap();

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert!(false);
}
