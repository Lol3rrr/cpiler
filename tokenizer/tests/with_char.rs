use general::{Source, Span};
use tokenizer::{Assignment, DataType, Keyword, Token, TokenData};

#[test]
fn simple_char() {
    let input_content = "char test = 'c';";
    let source = Source::new("test", input_content);
    let input_span: Span = source.clone().into();

    let expected: Vec<Token> = vec![
        Token {
            span: Span::new_source(source.clone(), 0..4),
            data: TokenData::Keyword(Keyword::DataType(DataType::Char)),
        },
        Token {
            span: Span::new_source(source.clone(), 5..9),
            data: TokenData::Literal {
                content: "test".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 10..11),
            data: TokenData::Assign(Assignment::Assign),
        },
        Token {
            span: Span::new_source(source.clone(), 12..15),
            data: TokenData::CharLiteral {
                content: "c".to_string(),
            },
        },
        Token {
            span: Span::new_source(source.clone(), 15..16),
            data: TokenData::Semicolon,
        },
    ];

    let result_iter = tokenizer::tokenize(input_span);
    let result_vec: Vec<_> = result_iter.collect();

    assert_eq!(expected, result_vec);
}
