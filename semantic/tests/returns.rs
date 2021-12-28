use general::{Source, Span};

#[test]
fn valid_void_function() {
    let content = "
void test() {
    int x = 13;
    if (x) {
        return;
    }

    int y = 4;
}
        ";
    let source = Source::new("test", content);
    let input_span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(tokens).unwrap();

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert_eq!(true, result.is_ok());
}

#[test]
fn invalid_void_function_in_if() {
    let content = "
void test() {
    int x = 13;
    if (x) {
        return 13;
    }

    int y = 4;

    return;
}
        ";
    let source = Source::new("test", content);
    let input_span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(tokens).unwrap();

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert_eq!(false, result.is_ok());
}

#[test]
fn valid_int_function() {
    let content = "
int test() {
    int x = 13;
    if (x) {
        return 13;
    }

    int y = 4;

    return 23;
}
        ";
    let source = Source::new("test", content);
    let input_span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(tokens).unwrap();

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert_eq!(true, result.is_ok());
}
