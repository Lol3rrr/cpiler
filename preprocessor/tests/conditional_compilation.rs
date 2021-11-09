use general::Span;
use preprocessor::loader::files::FileLoader;
use tokenizer::{DataType, Keyword, Token, TokenData};

#[test]
fn ifdef_conditional() {
    let file_name = "./tests/files/conditional_compilation/ifdef.c";
    let loader = FileLoader::new();

    let expected = vec![
        Token {
            span: Span::from_parts(file_name, "int", 26..29),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts(file_name, "first", 30..35),
            data: TokenData::Literal {
                content: "first".to_string(),
            },
        },
        Token {
            span: Span::from_parts(file_name, ";", 35..36),
            data: TokenData::Semicolon,
        },
    ];

    let result = preprocessor::preprocess(&loader, file_name).unwrap();

    assert_eq!(expected, result);
}

#[test]
fn nested_ifdefs() {
    let file_name = "./tests/files/conditional_compilation/nested_ifdefs.c";
    let loader = FileLoader::new();

    let expected = vec![
        Token {
            span: Span::from_parts(file_name, "int", 55..58),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts(file_name, "first", 59..64),
            data: TokenData::Literal {
                content: "first".to_string(),
            },
        },
        Token {
            span: Span::from_parts(file_name, ";", 64..65),
            data: TokenData::Semicolon,
        },
    ];

    let result = preprocessor::preprocess(&loader, file_name).unwrap();

    assert_eq!(expected, result);
}

#[test]
fn defined_condition() {
    let file_name = "./tests/files/conditional_compilation/defined_condition.c";
    let loader = FileLoader::new();

    let expected = vec![
        Token {
            span: Span::from_parts(file_name, "int", 29..32),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts(file_name, "first", 33..38),
            data: TokenData::Literal {
                content: "first".to_string(),
            },
        },
        Token {
            span: Span::from_parts(file_name, ";", 38..39),
            data: TokenData::Semicolon,
        },
    ];

    let result = preprocessor::preprocess(&loader, file_name).unwrap();

    assert_eq!(expected, result);
}
