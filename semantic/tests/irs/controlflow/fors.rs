use general::{Source, Span};

#[test]
#[ignore = "Does not support for loops yet"]
fn basic_for_loop() {
    let content = "
void test() {
    for (int i = 0; i < 10; i++) {
        int x = i;
    }
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    assert!(false);
}

#[test]
#[ignore = "For loops are not yet supported really"]
fn access_after_for_loop() {
    let content = "
void test() {
    int tmp = 0;

    for (int i = 0; i < 10; i++) {
        int x = i;
    }

    int other = tmp;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    assert!(false);
}
