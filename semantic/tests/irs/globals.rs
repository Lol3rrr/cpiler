use general::{Source, Span};

#[test]
#[ignore = "Figure out a better way to verify semantics"]
fn read_global() {
    let input = "
int global = 0;
void test() {
    int x = global;
    int y = global;
    int w = x + y;
}
        ";

    let input_source = Source::new("test", input);
    let input_span: Span = input_source.into();
    let input_tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(input_tokens).unwrap();

    let input = semantic::parse(input_ast).unwrap();

    let ir_result = input.convert_to_ir(general::arch::Arch::X86_64);

    println!("{}", ir::text_rep::program_text_rep(&ir_result));

    panic!();
}
