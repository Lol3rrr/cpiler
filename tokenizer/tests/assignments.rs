use general::{Source, Span};
use tokenizer::{Assignment, Token, TokenData};

#[test]
fn assignments() {
    let input_source = Source::new("test", "= += -= *= /= %= >>= <<= &= |= ^=");
    let input_span: Span = input_source.clone().into();

    let expected = vec![
        Token {
            span: Span::new_source(input_source.clone(), 0..1),
            data: TokenData::Assign(Assignment::Assign),
        },
        Token {
            span: Span::new_source(input_source.clone(), 2..4),
            data: TokenData::Assign(Assignment::Add),
        },
        Token {
            span: Span::new_source(input_source.clone(), 5..7),
            data: TokenData::Assign(Assignment::Sub),
        },
        Token {
            span: Span::new_source(input_source.clone(), 8..10),
            data: TokenData::Assign(Assignment::Multiply),
        },
        Token {
            span: Span::new_source(input_source.clone(), 11..13),
            data: TokenData::Assign(Assignment::Divide),
        },
        Token {
            span: Span::new_source(input_source.clone(), 14..16),
            data: TokenData::Assign(Assignment::Modulo),
        },
        Token {
            span: Span::new_source(input_source.clone(), 17..20),
            data: TokenData::Assign(Assignment::ShiftRight),
        },
        Token {
            span: Span::new_source(input_source.clone(), 21..24),
            data: TokenData::Assign(Assignment::ShiftLeft),
        },
        Token {
            span: Span::new_source(input_source.clone(), 25..27),
            data: TokenData::Assign(Assignment::BitwiseAnd),
        },
        Token {
            span: Span::new_source(input_source.clone(), 28..30),
            data: TokenData::Assign(Assignment::BitwiseOr),
        },
        Token {
            span: Span::new_source(input_source.clone(), 31..33),
            data: TokenData::Assign(Assignment::BitwiseXor),
        },
    ];

    let result: Vec<_> = tokenizer::tokenize(input_span).collect();

    dbg!(&result);

    assert_eq!(expected, result);
}
