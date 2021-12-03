use general::{Source, Span, SpanData};
use syntax::{EnumVariant, EnumVariants, Identifier, Scope, Statement, AST};

#[test]
fn basic_named_enum() {
    let input = "
enum test {
    first,
    second,
    third,
};
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let mut tokens = tokenizer::tokenize(span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::EnumDefinition {
                name: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 6..10),
                    data: "test".to_string(),
                }),
                variants: EnumVariants {
                    members: vec![
                        EnumVariant {
                            name: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 17..22),
                                data: "first".to_string(),
                            }),
                            value: None,
                        },
                        EnumVariant {
                            name: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 28..34),
                                data: "second".to_string(),
                            }),
                            value: None,
                        },
                        EnumVariant {
                            name: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 40..45),
                                data: "third".to_string(),
                            }),
                            value: None,
                        },
                    ],
                },
            }],
        },
    });

    let result = syntax::parse(tokens.by_ref());
    dbg!(&result);

    assert_eq!(expected, result);
    assert_eq!(None, tokens.next());
}

#[test]
fn enum_with_assigned_values() {
    let input = "
enum test {
    first,
    second = 0,
    third,
};
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let mut tokens = tokenizer::tokenize(span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::EnumDefinition {
                name: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 6..10),
                    data: "test".to_string(),
                }),
                variants: EnumVariants {
                    members: vec![
                        EnumVariant {
                            name: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 17..22),
                                data: "first".to_string(),
                            }),
                            value: None,
                        },
                        EnumVariant {
                            name: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 28..34),
                                data: "second".to_string(),
                            }),
                            value: Some(SpanData {
                                span: Span::new_source(source.clone(), 37..38),
                                data: 0,
                            }),
                        },
                        EnumVariant {
                            name: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 44..49),
                                data: "third".to_string(),
                            }),
                            value: None,
                        },
                    ],
                },
            }],
        },
    });

    let result = syntax::parse(tokens.by_ref());
    dbg!(&result);

    assert_eq!(expected, result);
    assert_eq!(None, tokens.next());
}
