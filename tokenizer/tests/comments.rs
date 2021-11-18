use general::{Source, Span};
use tokenizer::{tokenize, ControlFlow, DataType, Keyword, Token, TokenData};

#[test]
fn comments_program() {
    let raw_content = include_str!("./files/comments.c");
    let source = Source::new("./files/comments.c", raw_content);
    let content: Span = source.clone().into();

    let expected: Vec<Token> = vec![
        Token {
            span: Span::new_source(source.clone(), 2..22),
            data: TokenData::Comment {
                content: " Comment before main".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 23..26),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(source.clone(), 27..31),
            data: TokenData::Literal {
                content: "main".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 31..32),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::new_source(source.clone(), 32..33),
            data: TokenData::CloseParen,
        },
        Token {
            span: Span::new_source(source.clone(), 34..35),
            data: TokenData::OpenBrace,
        },
        Token {
            span: Span::new_source(source.clone(), 39..55),
            data: TokenData::Comment {
                content: " Comment in main".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 57..63),
            data: TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)),
        },
        Token {
            span: Span::new_source(source.clone(), 64..65),
            data: TokenData::Literal {
                content: "0".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 65..66),
            data: TokenData::Semicolon,
        },
        Token {
            span: Span::new_source(source.clone(), 70..112),
            data: TokenData::Comment {
                content: "\n\t * Multi line comment\n\t * second line\n\t ".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 115..116),
            data: TokenData::CloseBrace,
        },
    ];

    let tokenized: Vec<_> = tokenize(content).collect();

    assert_eq!(expected, tokenized);
}
