use std::sync::Arc;

use general::{Source, Span};
use preprocessor::loader::files::FileLoader;
use tokenizer::{DataType, Keyword, Token, TokenData};

#[test]
fn ifdef_conditional() {
    let file_name = "./tests/files/conditional_compilation/ifdef.c";
    let loader = FileLoader::new();

    let source = Source::new(
        file_name,
        include_str!("./files/conditional_compilation/ifdef.c"),
    );

    let expected = vec![
        Token {
            span: Span::new_source(source.clone(), 26..29),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(source.clone(), 30..35),
            data: TokenData::Literal {
                content: "first".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 35..36),
            data: TokenData::Semicolon,
        },
    ];

    let result = preprocessor::preprocess(Arc::new(loader), file_name).unwrap();

    assert_eq!(expected, result);
}

#[test]
fn ifndef_conditional() {
    let file_name = "./tests/files/conditional_compilation/ifndef.c";
    let loader = FileLoader::new();

    let source = Source::new(
        file_name,
        include_str!("./files/conditional_compilation/ifndef.c"),
    );

    let expected = vec![
        Token {
            span: Span::new_source(source.clone(), 12..15),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(source.clone(), 16..21),
            data: TokenData::Literal {
                content: "first".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 21..22),
            data: TokenData::Semicolon,
        },
    ];

    let result = preprocessor::preprocess(Arc::new(loader), file_name).unwrap();

    assert_eq!(expected, result);
}

#[test]
fn nested_ifdefs() {
    let file_name = "./tests/files/conditional_compilation/nested_ifdefs.c";
    let loader = FileLoader::new();

    let source = Source::new(
        file_name,
        include_str!("./files/conditional_compilation/nested_ifdefs.c"),
    );

    let expected = vec![
        Token {
            span: Span::new_source(source.clone(), 55..58),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(source.clone(), 59..64),
            data: TokenData::Literal {
                content: "first".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 64..65),
            data: TokenData::Semicolon,
        },
    ];

    let result = preprocessor::preprocess(Arc::new(loader), file_name).unwrap();

    assert_eq!(expected, result);
}

#[test]
fn defined_condition() {
    let file_name = "./tests/files/conditional_compilation/defined_condition.c";
    let loader = FileLoader::new();

    let source = Source::new(
        file_name,
        include_str!("./files/conditional_compilation/defined_condition.c"),
    );

    let expected = vec![
        Token {
            span: Span::new_source(source.clone(), 48..51),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(source.clone(), 52..57),
            data: TokenData::Literal {
                content: "first".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 57..58),
            data: TokenData::Semicolon,
        },
    ];

    let result = preprocessor::preprocess(Arc::new(loader), file_name).unwrap();

    assert_eq!(expected, result);
}

#[test]
fn condition_else() {
    let file_name = "./tests/files/conditional_compilation/condition_else.c";
    let loader = FileLoader::new();

    let source = Source::new(
        file_name,
        include_str!("./files/conditional_compilation/condition_else.c"),
    );

    let expected = vec![
        Token {
            span: Span::new_source(source.clone(), 27..30),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(source.clone(), 31..37),
            data: TokenData::Literal {
                content: "second".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 37..38),
            data: TokenData::Semicolon,
        },
    ];

    let result = preprocessor::preprocess(Arc::new(loader), file_name).unwrap();

    assert_eq!(expected, result);
}

#[test]
fn condition_ifelse_true() {
    let file_name = "./tests/files/conditional_compilation/condition_else_if_true.c";
    let loader = FileLoader::new();

    let source = Source::new(
        file_name,
        include_str!("./files/conditional_compilation/condition_else_if_true.c"),
    );

    let expected = vec![
        Token {
            span: Span::new_source(source.clone(), 34..37),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(source.clone(), 38..44),
            data: TokenData::Literal {
                content: "second".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 44..45),
            data: TokenData::Semicolon,
        },
    ];

    let result = preprocessor::preprocess(Arc::new(loader), file_name).unwrap();

    assert_eq!(expected, result);
}
