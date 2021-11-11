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
fn ifndef_conditional() {
    let file_name = "./tests/files/conditional_compilation/ifndef.c";
    let loader = FileLoader::new();

    let expected = vec![
        Token {
            span: Span::from_parts(file_name, "int", 12..15),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts(file_name, "first", 16..21),
            data: TokenData::Literal {
                content: "first".to_string(),
            },
        },
        Token {
            span: Span::from_parts(file_name, ";", 21..22),
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

#[test]
fn condition_else() {
    let file_name = "./tests/files/conditional_compilation/condition_else.c";
    let loader = FileLoader::new();

    let expected = vec![
        Token {
            span: Span::from_parts(file_name, "int", 27..30),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts(file_name, "second", 31..37),
            data: TokenData::Literal {
                content: "second".to_string(),
            },
        },
        Token {
            span: Span::from_parts(file_name, ";", 37..38),
            data: TokenData::Semicolon,
        },
    ];

    let result = preprocessor::preprocess(&loader, file_name).unwrap();

    assert_eq!(expected, result);
}

#[test]
fn condition_ifelse_true() {
    let file_name = "./tests/files/conditional_compilation/condition_else_if_true.c";
    let loader = FileLoader::new();

    let expected = vec![
        Token {
            span: Span::from_parts(file_name, "int", 34..37),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts(file_name, "second", 38..44),
            data: TokenData::Literal {
                content: "second".to_string(),
            },
        },
        Token {
            span: Span::from_parts(file_name, ";", 44..45),
            data: TokenData::Semicolon,
        },
    ];

    let result = preprocessor::preprocess(&loader, file_name).unwrap();

    assert_eq!(expected, result);
}
