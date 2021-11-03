use general::Span;
use preprocessor::loader::files::FileLoader;
use tokenizer::{ControlFlow, DataType, Keyword, Token, TokenData};

#[test]
fn simple_define() {
    let loader = FileLoader::new();

    let expected = vec![
        Token {
            span: Span::from_parts("./tests/files/define-block.c", "int", 16..19),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts("./tests/files/define-block.c", "main", 20..24),
            data: TokenData::Literal {
                content: "main".to_owned(),
            },
        },
        Token {
            span: Span::from_parts("./tests/files/define-block.c", "(", 24..25),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::from_parts("./tests/files/define-block.c", ")", 25..26),
            data: TokenData::CloseParen,
        },
        Token {
            span: Span::from_parts("./tests/files/define-block.c", "{", 27..28),
            data: TokenData::OpenBrace,
        },
        Token {
            span: Span::from_parts("./tests/files/define-block.c", "return", 30..36),
            data: TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)),
        },
        Token {
            span: Span::from_parts("./tests/files/define-block.c", "0", 13..14),
            data: TokenData::Literal {
                content: "0".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./tests/files/define-block.c", ";", 41..42),
            data: TokenData::Semicolon,
        },
        Token {
            span: Span::from_parts("./tests/files/define-block.c", "}", 43..44),
            data: TokenData::CloseBrace,
        },
    ];

    let result = preprocessor::preprocess(&loader, "./tests/files/define-block.c").unwrap();

    assert_eq!(expected, result);
}
