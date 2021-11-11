use general::Span;
use tokenizer::{Assignment, Token, TokenData};

#[test]
fn assignments() {
    let input_content = "= += -= *= /= %= >>= <<= &= |= ^=";
    let input_span = Span::from_parts("test", input_content, 0..input_content.len());

    let expected = vec![
        Token {
            span: Span::from_parts("test", "=", 0..1),
            data: TokenData::Assign(Assignment::Assign),
        },
        Token {
            span: Span::from_parts("test", "+=", 2..4),
            data: TokenData::Assign(Assignment::Add),
        },
        Token {
            span: Span::from_parts("test", "-=", 5..7),
            data: TokenData::Assign(Assignment::Sub),
        },
        Token {
            span: Span::from_parts("test", "*=", 8..10),
            data: TokenData::Assign(Assignment::Multiply),
        },
        Token {
            span: Span::from_parts("test", "/=", 11..13),
            data: TokenData::Assign(Assignment::Divide),
        },
        Token {
            span: Span::from_parts("test", "%=", 14..16),
            data: TokenData::Assign(Assignment::Modulo),
        },
        Token {
            span: Span::from_parts("test", ">>=", 17..20),
            data: TokenData::Assign(Assignment::ShiftRight),
        },
        Token {
            span: Span::from_parts("test", "<<=", 21..24),
            data: TokenData::Assign(Assignment::ShiftLeft),
        },
        Token {
            span: Span::from_parts("test", "&=", 25..27),
            data: TokenData::Assign(Assignment::BitwiseAnd),
        },
        Token {
            span: Span::from_parts("test", "|=", 28..30),
            data: TokenData::Assign(Assignment::BitwiseOr),
        },
        Token {
            span: Span::from_parts("test", "^=", 31..33),
            data: TokenData::Assign(Assignment::BitwiseXor),
        },
    ];

    let result: Vec<_> = tokenizer::tokenize(input_span).collect();

    dbg!(&result);

    assert_eq!(expected, result);
}
