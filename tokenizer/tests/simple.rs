use general::Span;
use tokenizer::{tokenize, ControlFlow, DataType, Keyword, Token, TokenData};

#[test]
fn simple_program() {
    let raw_content = include_str!("./files/simple.c");
    let content = Span::new_source("./files/simple.c", raw_content);

    let expected: Vec<Token> = vec![
        Token {
            span: Span::from_parts("./files/simple.c", "int", 0..3),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts("./files/simple.c", "main", 4..8),
            data: TokenData::Literal {
                content: "main".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./files/simple.c", "(", 8..9),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::from_parts("./files/simple.c", ")", 9..10),
            data: TokenData::CloseParen,
        },
        Token {
            span: Span::from_parts("./files/simple.c", "{", 11..12),
            data: TokenData::OpenBrace,
        },
        Token {
            span: Span::from_parts("./files/simple.c", "return", 14..20),
            data: TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)),
        },
        Token {
            span: Span::from_parts("./files/simple.c", "0", 21..22),
            data: TokenData::Literal {
                content: "0".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./files/simple.c", ";", 22..23),
            data: TokenData::Semicolon,
        },
        Token {
            span: Span::from_parts("./files/simple.c", "}", 24..25),
            data: TokenData::CloseBrace,
        },
    ];

    let tokenized = tokenize(content);

    assert_eq!(expected, tokenized);
}
