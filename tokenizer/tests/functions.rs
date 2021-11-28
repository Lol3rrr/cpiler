use general::{Source, Span};
use tokenizer::{tokenize, DataType, Keyword, Operator, Token, TokenData};

#[test]
fn func_dec() {
    let content = "void test(int *test)";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();

    let expected = vec![
        Token {
            span: Span::new_source(source.clone(), 0..4),
            data: TokenData::Keyword(Keyword::DataType(DataType::Void)),
        },
        Token {
            span: Span::new_source(source.clone(), 5..9),
            data: TokenData::Literal {
                content: "test".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 9..10),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::new_source(source.clone(), 10..13),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(source.clone(), 14..15),
            data: TokenData::Operator(Operator::Multiply),
        },
        Token {
            span: Span::new_source(source.clone(), 15..19),
            data: TokenData::Literal {
                content: "test".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 19..20),
            data: TokenData::CloseParen,
        },
    ];

    let result_iter = tokenizer::tokenize(span);
    let result_vec: Vec<_> = result_iter.collect();

    dbg!(&result_vec);

    assert_eq!(expected, result_vec);
}

#[test]
fn func_var_arg() {
    let content = "void test(int first, ...)";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();

    let expected = vec![
        Token {
            span: Span::new_source(source.clone(), 0..4),
            data: TokenData::Keyword(Keyword::DataType(DataType::Void)),
        },
        Token {
            span: Span::new_source(source.clone(), 5..9),
            data: TokenData::Literal {
                content: "test".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 9..10),
            data: TokenData::OpenParen,
        },
        Token {
            span: Span::new_source(source.clone(), 10..13),
            data: TokenData::Keyword(Keyword::DataType(DataType::Int)),
        },
        Token {
            span: Span::new_source(source.clone(), 14..19),
            data: TokenData::Literal {
                content: "first".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 19..20),
            data: TokenData::Comma,
        },
        Token {
            span: Span::new_source(source.clone(), 21..24),
            data: TokenData::VarArgs,
        },
        Token {
            span: Span::new_source(source.clone(), 24..25),
            data: TokenData::CloseParen,
        },
    ];

    let result_iter = tokenizer::tokenize(span);
    let result_vec: Vec<_> = result_iter.collect();

    dbg!(&result_vec);

    assert_eq!(expected, result_vec);
}
