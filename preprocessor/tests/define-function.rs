use general::Span;
use preprocessor::loader::files::FileLoader;
use tokenizer::{ControlFlow, DataType, Keyword, Operator, Token, TokenData};

#[test]
fn simple_function_define() {
    let file = "./tests/files/define-function.c";

    let loader = FileLoader::new();

    let expected = vec![
        Token {
            span: Span::from_parts(file, "int", 25..28),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::from_parts(file, "main", 29..33),
            data: TokenData::Literal {
                content: "main".to_string(),
            },
        },
        Token {
            span: Span::from_parts(file, "(", 33..34),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::from_parts(file, ")", 34..35),
            data: TokenData::CloseParen,
        },
        Token {
            span: Span::from_parts(file, "{", 36..37),
            data: TokenData::OpenBrace,
        },
        Token {
            span: Span::from_parts(file, "return", 39..45),
            data: TokenData::Keyword(Keyword::ControlFlow(ControlFlow::Return)),
        },
        Token {
            span: Span::from_parts(file, "(", 16..17),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::from_parts(file, "-", 51..52),
            data: TokenData::Operator(Operator::Sub),
        },
        Token {
            span: Span::from_parts(file, "1", 52..53),
            data: TokenData::Literal {
                content: "1".to_string(),
            },
        },
        Token {
            span: Span::from_parts(file, "+", 19..20),
            data: TokenData::Operator(Operator::Add),
        },
        Token {
            span: Span::from_parts(file, "1", 21..22),
            data: TokenData::Literal {
                content: "1".to_string(),
            },
        },
        Token {
            span: Span::from_parts(file, ")", 22..23),
            data: TokenData::CloseParen,
        },
        Token {
            span: Span::from_parts(file, ";", 54..55),
            data: TokenData::Semicolon,
        },
        Token {
            span: Span::from_parts(file, "}", 56..57),
            data: TokenData::CloseBrace,
        },
    ];

    let result = preprocessor::preprocess(&loader, file).unwrap();

    assert_eq!(expected, result);
}
