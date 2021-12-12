use std::path::{Path, PathBuf};

use general::{Source, Span};

#[test]
fn simple_while_loop() {
    let input = "
void test() {
    while (2) {
        int x = 0;
    }

    return;
}
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let result = input.convert_to_ir();
    dbg!(&result);

    let test_file_path = {
        let raw = Path::new(file!());
        let mut result = PathBuf::new();
        for part in raw.components().skip(1) {
            result.push(part);
        }

        result
    };
    let test_dir_path = test_file_path.parent().unwrap();
    dbg!(&test_dir_path);

    let result_path = test_dir_path.join("result.dot");
    dbg!(&result_path);
    std::fs::write(result_path, result.to_dot()).unwrap();

    assert!(false);
}
