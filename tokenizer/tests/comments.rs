use general::Span;
use tokenizer::{tokenize, ControlFlow, DataType, Keyword, Token, TokenData};

#[test]
fn comments_program() {
    let raw_content = include_str!("./files/comments.c");
    let content = Span::new_source("./files/comments.c", raw_content);

    let expected: Vec<Token> = vec![
        Token {
            span: Span::from_parts("./files/comments.c", " Comment before main", 2..22),
            data: TokenData::Comment {
                content: " Comment before main".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./files/comments.c", "int", 23..26),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts("./files/comments.c", "main", 27..31),
            data: TokenData::Literal {
                content: "main".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./files/comments.c", "(", 31..32),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::from_parts("./files/comments.c", ")", 32..33),
            data: TokenData::CloseParen,
        },
        Token {
            span: Span::from_parts("./files/comments.c", "{", 34..35),
            data: TokenData::OpenBrace,
        },
        Token {
            span: Span::from_parts("./files/comments.c", " Comment in main", 39..55),
            data: TokenData::Comment {
                content: " Comment in main".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./files/comments.c", "return", 57..63),
            data: TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)),
        },
        Token {
            span: Span::from_parts("./files/comments.c", "0", 64..65),
            data: TokenData::Literal {
                content: "0".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./files/comments.c", ";", 65..66),
            data: TokenData::Semicolon,
        },
        Token {
            span: Span::from_parts(
                "./files/comments.c",
                "\n\t * Multi line comment\n\t * second line\n\t ",
                70..112,
            ),
            data: TokenData::Comment {
                content: "\n\t * Multi line comment\n\t * second line\n\t ".to_string(),
            },
        },
        Token {
            span: Span::from_parts("./files/comments.c", "}", 115..116),
            data: TokenData::CloseBrace,
        },
    ];

    let tokenized = tokenize(content);

    assert_eq!(expected, tokenized);
}
