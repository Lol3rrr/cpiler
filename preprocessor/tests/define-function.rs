use std::sync::Arc;

use general::{Source, Span};
use preprocessor::loader::files::FileLoader;
use tokenizer::{ControlFlow, DataType, Keyword, Operator, Token, TokenData};

#[test]
fn simple_function_define() {
    let file = "./tests/files/define-function.c";

    let loader = FileLoader::new();

    let define_source = Source::new(file, include_str!("./files/define-function.c"));
    let arced_source = Arc::new(define_source.clone());

    let expected = vec![
        Token {
            span: Span::new_source(define_source.clone(), 25..28),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(define_source.clone(), 29..33),
            data: TokenData::Literal {
                content: "main".to_string(),
            },
        },
        Token {
            span: Span::new_source(define_source.clone(), 33..34),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::new_source(define_source.clone(), 34..35),
            data: TokenData::CloseParen,
        },
        Token {
            span: Span::new_source(define_source.clone(), 36..37),
            data: TokenData::OpenBrace,
        },
        Token {
            span: Span::new_source(define_source.clone(), 39..45),
            data: TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)),
        },
        Token {
            span: Span::new_arc_source_og(
                arced_source.clone(),
                46..50,
                Span::new_source(define_source.clone(), 16..17),
            ),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::new_source(define_source.clone(), 51..52),
            data: TokenData::Operator(Operator::Sub),
        },
        Token {
            span: Span::new_source(define_source.clone(), 52..53),
            data: TokenData::Literal {
                content: "1".to_string(),
            },
        },
        Token {
            span: Span::new_arc_source_og(
                arced_source.clone(),
                46..50,
                Span::new_source(define_source.clone(), 19..20),
            ),
            data: TokenData::Operator(Operator::Add),
        },
        Token {
            span: Span::new_arc_source_og(
                arced_source.clone(),
                46..50,
                Span::new_source(define_source.clone(), 21..22),
            ),
            data: TokenData::Literal {
                content: "1".to_string(),
            },
        },
        Token {
            span: Span::new_arc_source_og(
                arced_source.clone(),
                46..50,
                Span::new_source(define_source.clone(), 22..23),
            ),
            data: TokenData::CloseParen,
        },
        Token {
            span: Span::new_source(define_source.clone(), 54..55),
            data: TokenData::Semicolon,
        },
        Token {
            span: Span::new_source(define_source.clone(), 56..57),
            data: TokenData::CloseBrace,
        },
    ];

    let result = preprocessor::preprocess(&loader, file).unwrap();

    dbg!(&result);

    assert_eq!(expected, result);
}
