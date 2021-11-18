use general::{Source, Span};
use tokenizer::{tokenize, Assignment, ControlFlow, DataType, Keyword, Operator, Token, TokenData};

#[test]
fn with_string_program() {
    let raw_content = include_str!("./files/with_string.c");
    let source = Source::new("./files/with_string.c", raw_content);
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
            span: Span::new_source(source.clone(), 14..18),
            data: TokenData::Keyword(Keyword::DataType(DataType::Char)),
        },
        Token {
            span: Span::new_source(source.clone(), 18..19),
            data: TokenData::Operator(Operator::Multiply),
        },
        Token {
            span: Span::new_source(source.clone(), 20..23),
            data: TokenData::Literal {
                content: "tmp".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 24..25),
            data: TokenData::Assign(Assignment::Assign),
        },
        Token {
            span: Span::new_source(source.clone(), 27..38),
            data: TokenData::StringLiteral {
                content: "test string".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 39..40),
            data: TokenData::Semicolon,
        },
        Token {
            span: Span::new_source(source.clone(), 43..49),
            data: TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)),
        },
        Token {
            span: Span::new_source(source.clone(), 50..51),
            data: TokenData::Literal {
                content: "0".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 51..52),
            data: TokenData::Semicolon,
        },
        Token {
            span: Span::new_source(source.clone(), 53..54),
            data: TokenData::CloseBrace,
        },
    ];

    let tokenized: Vec<_> = tokenize(content).collect();

    assert_eq!(expected, tokenized);
}
