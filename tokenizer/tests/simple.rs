use general::{Source, Span};
use tokenizer::{tokenize, ControlFlow, DataType, Keyword, Token, TokenData};

#[test]
fn simple_program() {
    let raw_content = include_str!("./files/simple.c");
    let source = Source::new("./files/simple.c", raw_content);
    let content: Span = source.clone().into();

    let expected: Vec<Token> = vec![
        Token {
            span: Span::new_source(source.clone(), 0..3),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(source.clone(), 4..8),
            data: TokenData::Literal {
                content: "main".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 8..9),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::new_source(source.clone(), 9..10),
            data: TokenData::CloseParen,
        },
        Token {
            span: Span::new_source(source.clone(), 11..12),
            data: TokenData::OpenBrace,
        },
        Token {
            span: Span::new_source(source.clone(), 14..20),
            data: TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)),
        },
        Token {
            span: Span::new_source(source.clone(), 21..22),
            data: TokenData::Literal {
                content: "0".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 22..23),
            data: TokenData::Semicolon,
        },
        Token {
            span: Span::new_source(source, 24..25),
            data: TokenData::CloseBrace,
        },
    ];

    let tokenized: Vec<_> = tokenize(content).collect();

    assert_eq!(expected, tokenized);
}
