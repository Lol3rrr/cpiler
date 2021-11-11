use general::Span;
use tokenizer::{Operator, Token, TokenData};

#[test]
fn operators() {
    let input = "-> . ++ -- + - ! ~ * & / % << >> < <= >= > == != ^ | && ||";
    let input_span = Span::from_parts("test", input, 0..input.len());

    let expected = vec![
        Token {
            span: Span::from_parts("test", "->", 0..2),
            data: TokenData::Operator(Operator::Arrow),
        },
        Token {
            span: Span::from_parts("test", ".", 3..4),
            data: TokenData::Operator(Operator::Dot),
        },
        Token {
            span: Span::from_parts("test", "++", 5..7),
            data: TokenData::Operator(Operator::Increment),
        },
        Token {
            span: Span::from_parts("test", "--", 8..10),
            data: TokenData::Operator(Operator::Decrement),
        },
        Token {
            span: Span::from_parts("test", "+", 11..12),
            data: TokenData::Operator(Operator::Add),
        },
        Token {
            span: Span::from_parts("test", "-", 13..14),
            data: TokenData::Operator(Operator::Sub),
        },
        Token {
            span: Span::from_parts("test", "!", 15..16),
            data: TokenData::Operator(Operator::LogicalNot),
        },
        Token {
            span: Span::from_parts("test", "~", 17..18),
            data: TokenData::Operator(Operator::BitwiseNot),
        },
        Token {
            span: Span::from_parts("test", "*", 19..20),
            data: TokenData::Operator(Operator::Multiply),
        },
        Token {
            span: Span::from_parts("test", "&", 21..22),
            data: TokenData::Operator(Operator::BitwiseAnd),
        },
        Token {
            span: Span::from_parts("test", "/", 23..24),
            data: TokenData::Operator(Operator::Divide),
        },
        Token {
            span: Span::from_parts("test", "%", 25..26),
            data: TokenData::Operator(Operator::Modulo),
        },
        Token {
            span: Span::from_parts("test", "<<", 27..29),
            data: TokenData::Operator(Operator::ShiftLeft),
        },
        Token {
            span: Span::from_parts("test", ">>", 30..32),
            data: TokenData::Operator(Operator::ShiftRight),
        },
        Token {
            span: Span::from_parts("test", "<", 33..34),
            data: TokenData::Operator(Operator::Less),
        },
        Token {
            span: Span::from_parts("test", "<=", 35..37),
            data: TokenData::Operator(Operator::LessEqual),
        },
        Token {
            span: Span::from_parts("test", ">=", 38..40),
            data: TokenData::Operator(Operator::GreaterEqual),
        },
        Token {
            span: Span::from_parts("test", ">", 41..42),
            data: TokenData::Operator(Operator::Greater),
        },
        Token {
            span: Span::from_parts("test", "==", 43..45),
            data: TokenData::Operator(Operator::Equal),
        },
        Token {
            span: Span::from_parts("test", "!=", 46..48),
            data: TokenData::Operator(Operator::NotEqual),
        },
        Token {
            span: Span::from_parts("test", "^", 49..50),
            data: TokenData::Operator(Operator::BitwiseXor),
        },
        Token {
            span: Span::from_parts("test", "|", 51..52),
            data: TokenData::Operator(Operator::BitwiseOr),
        },
        Token {
            span: Span::from_parts("test", "&&", 53..55),
            data: TokenData::Operator(Operator::LogicalAnd),
        },
        Token {
            span: Span::from_parts("test", "||", 56..58),
            data: TokenData::Operator(Operator::LogicalOr),
        },
    ];

    let result: Vec<_> = tokenizer::tokenize(input_span).collect();

    dbg!(&result);

    assert_eq!(expected, result);
}
