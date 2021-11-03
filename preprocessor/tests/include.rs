use general::Span;
use preprocessor::loader::files::FileLoader;
use tokenizer::{Assignment, ControlFlow, DataType, Keyword, Token, TokenData};

#[test]
fn simple_include() {
    let loader = FileLoader::new();

    let expected = vec![
        Token {
            span: Span::from_parts("./tests/files/other.c", "int", 0..3),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts("./tests/files/other.c", "test", 4..8),
            data: TokenData::Literal {
                content: "test".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./tests/files/other.c", "=", 9..10),
            data: TokenData::Assign(Assignment::Assign),
        },
        Token {
            span: Span::from_parts("./tests/files/other.c", "0", 11..12),
            data: TokenData::Literal {
                content: "0".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./tests/files/other.c", ";", 12..13),
            data: TokenData::Semicolon,
        },
        // The Data of the "original File"
        Token {
            span: Span::from_parts("./tests/files/include.c", "int", 20..23),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts("./tests/files/include.c", "main", 24..28),
            data: TokenData::Literal {
                content: "main".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./tests/files/include.c", "(", 28..29),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::from_parts("./tests/files/include.c", ")", 29..30),
            data: TokenData::CloseParen,
        },
        Token {
            span: Span::from_parts("./tests/files/include.c", "{", 31..32),
            data: TokenData::OpenBrace,
        },
        Token {
            span: Span::from_parts("./tests/files/include.c", "return", 34..40),
            data: TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)),
        },
        Token {
            span: Span::from_parts("./tests/files/include.c", "0", 41..42),
            data: TokenData::Literal {
                content: "0".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./tests/files/include.c", ";", 42..43),
            data: TokenData::Semicolon,
        },
        Token {
            span: Span::from_parts("./tests/files/include.c", "}", 44..45),
            data: TokenData::CloseBrace,
        },
    ];

    let result = preprocessor::preprocess(&loader, "./tests/files/include.c").unwrap();

    assert_eq!(expected, result);
}
