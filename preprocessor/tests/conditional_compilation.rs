use general::Span;
use preprocessor::loader::files::FileLoader;
use tokenizer::{DataType, Keyword, Token, TokenData};

#[test]
fn simple_conditional() {
    let file_name = "./tests/files/conditional_compilation/simple.c";
    let loader = FileLoader::new();

    let expected = vec![
        Token {
            span: Span::from_parts(file_name, "int", 0..0),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts(file_name, "first", 0..0),
            data: TokenData::Literal {
                content: "first".to_string(),
            },
        },
    ];

    let result = preprocessor::preprocess(&loader, file_name).unwrap();

    assert_eq!(expected, result);
}
