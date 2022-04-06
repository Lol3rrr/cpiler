use std::sync::Arc;

use general::{Source, Span};
use preprocessor::loader::files::FileLoader;
use tokenizer::{ControlFlow, DataType, Keyword, Token, TokenData};

#[test]
fn simple_define() {
    let loader = FileLoader::new();

    let define_source = Source::new(
        "./tests/files/define-block.c",
        include_str!("./files/define-block.c"),
    );

    let expected = vec![
        Token {
            span: Span::new_source(define_source.clone(), 16..19),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(define_source.clone(), 20..24),
            data: TokenData::Literal {
                content: "main".to_owned(),
            },
        },
        Token {
            span: Span::new_source(define_source.clone(), 24..25),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::new_source(define_source.clone(), 25..26),
            data: TokenData::CloseParen,
        },
        Token {
            span: Span::new_source(define_source.clone(), 27..28),
            data: TokenData::OpenBrace,
        },
        Token {
            span: Span::new_source(define_source.clone(), 30..36),
            data: TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)),
        },
        Token {
            span: Span::new_arc_source_og(
                Arc::new(define_source.clone()),
                37..41,
                Span::new_source(define_source.clone(), 13..14),
            ),
            data: TokenData::Literal {
                content: "0".to_string(),
            },
        },
        Token {
            span: Span::new_source(define_source.clone(), 41..42),
            data: TokenData::Semicolon,
        },
        Token {
            span: Span::new_source(define_source, 43..44),
            data: TokenData::CloseBrace,
        },
    ];

    let result =
        preprocessor::preprocess(Arc::new(loader), "./tests/files/define-block.c").unwrap();

    assert_eq!(expected, result);
}
