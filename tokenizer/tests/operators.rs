use general::{Source, Span};
use tokenizer::{Operator, Token, TokenData};

#[test]
fn operators() {
    let input = "-> . ++ -- + - ! ~ * & / % << >> < <= >= > == != ^ | && ||";
    let source = Source::new("test", input);
    let input_span: Span = source.clone().into();

    let expected = vec![
        Token {
            span: Span::new_source(source.clone(), 0..2),
            data: TokenData::Operator(Operator::Arrow),
        },
        Token {
            span: Span::new_source(source.clone(), 3..4),
            data: TokenData::Operator(Operator::Dot),
        },
        Token {
            span: Span::new_source(source.clone(), 5..7),
            data: TokenData::Operator(Operator::Increment),
        },
        Token {
            span: Span::new_source(source.clone(), 8..10),
            data: TokenData::Operator(Operator::Decrement),
        },
        Token {
            span: Span::new_source(source.clone(), 11..12),
            data: TokenData::Operator(Operator::Add),
        },
        Token {
            span: Span::new_source(source.clone(), 13..14),
            data: TokenData::Operator(Operator::Sub),
        },
        Token {
            span: Span::new_source(source.clone(), 15..16),
            data: TokenData::Operator(Operator::LogicalNot),
        },
        Token {
            span: Span::new_source(source.clone(), 17..18),
            data: TokenData::Operator(Operator::BitwiseNot),
        },
        Token {
            span: Span::new_source(source.clone(), 19..20),
            data: TokenData::Operator(Operator::Multiply),
        },
        Token {
            span: Span::new_source(source.clone(), 21..22),
            data: TokenData::Operator(Operator::BitwiseAnd),
        },
        Token {
            span: Span::new_source(source.clone(), 23..24),
            data: TokenData::Operator(Operator::Divide),
        },
        Token {
            span: Span::new_source(source.clone(), 25..26),
            data: TokenData::Operator(Operator::Modulo),
        },
        Token {
            span: Span::new_source(source.clone(), 27..29),
            data: TokenData::Operator(Operator::ShiftLeft),
        },
        Token {
            span: Span::new_source(source.clone(), 30..32),
            data: TokenData::Operator(Operator::ShiftRight),
        },
        Token {
            span: Span::new_source(source.clone(), 33..34),
            data: TokenData::Operator(Operator::Less),
        },
        Token {
            span: Span::new_source(source.clone(), 35..37),
            data: TokenData::Operator(Operator::LessEqual),
        },
        Token {
            span: Span::new_source(source.clone(), 38..40),
            data: TokenData::Operator(Operator::GreaterEqual),
        },
        Token {
            span: Span::new_source(source.clone(), 41..42),
            data: TokenData::Operator(Operator::Greater),
        },
        Token {
            span: Span::new_source(source.clone(), 43..45),
            data: TokenData::Operator(Operator::Equal),
        },
        Token {
            span: Span::new_source(source.clone(), 46..48),
            data: TokenData::Operator(Operator::NotEqual),
        },
        Token {
            span: Span::new_source(source.clone(), 49..50),
            data: TokenData::Operator(Operator::BitwiseXor),
        },
        Token {
            span: Span::new_source(source.clone(), 51..52),
            data: TokenData::Operator(Operator::BitwiseOr),
        },
        Token {
            span: Span::new_source(source.clone(), 53..55),
            data: TokenData::Operator(Operator::LogicalAnd),
        },
        Token {
            span: Span::new_source(source.clone(), 56..58),
            data: TokenData::Operator(Operator::LogicalOr),
        },
    ];

    let result: Vec<_> = tokenizer::tokenize(input_span).collect();

    dbg!(&result);

    assert_eq!(expected, result);
}
