use general::{Source, Span};
use preprocessor::loader::files::FileLoader;
use tokenizer::{Token, TokenData};

#[test]
fn nested_defines() {
    let define_file: &str = "./tests/files/define.c";

    let loader = FileLoader::new();

    let define_source = Source::new(define_file, include_str!("./files/define.c"));

    let expected = vec![
        Token {
            span: Span::new_source(define_source.clone(), 48..56),
            data: TokenData::StringLiteral {
                content: "HI THERE".to_owned(),
            },
        },
        Token {
            span: Span::new_source(define_source.clone(),
                151..216,
            ),
            data: TokenData::Comment {
                content: " \"HI THERE\", because concatenation occurs before normal expansion"
                    .to_owned(),
            },
        },
        Token {
            span: Span::new_source(Source::new("preprocessor", "HI_THERE"), 0..8),
            data: TokenData::Literal {
                content: "HI_THERE".to_owned(),
            },
        },
        Token {
            span: Span::new_source(define_source.clone(), 233..326),
            data: TokenData::Comment {
                content: " HI_THERE, because the tokens originating from parameters (\"HE\" and \"LLO\") are expanded first".to_string(),
            },
        },
        Token {
            span: Span::new_source(define_source.clone(), 48..56),
            data: TokenData::StringLiteral {
                content: "HI THERE".to_owned(),
            },
        },
        Token {
            span: Span::new_source(define_source.clone(), 339..389),
            data: TokenData::Comment {
                content: " \"HI THERE\", because parameters are expanded first".to_owned(),
            },
        },
    ];

    let result = preprocessor::preprocess(&loader, define_file).unwrap();

    dbg!(&result);

    assert_eq!(expected, result);
}
