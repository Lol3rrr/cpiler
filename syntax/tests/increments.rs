use general::{Source, Span, SpanData};
use syntax::{Expression, Identifier, SingleOperation, Statement, AST};

#[test]
fn literals() {
    let content = "
    tmp++;
    ++tmp;
    tmp--;
    --tmp;
        ";

    let source = Source::new("test", content);
    let input_span: Span = source.clone().into();
    let input_tokenized = tokenizer::tokenize(input_span);

    let expected = Ok(AST {
        global_scope: syntax::Scope {
            statements: vec![
                Statement::SingleExpression(Expression::SingleOperation {
                    base: Box::new(Expression::Identifier {
                        ident: Identifier(SpanData {
                            span: Span::new_source(source.clone(), 5..8),
                            data: "tmp".to_string(),
                        }),
                    }),
                    operation: SingleOperation::SuffixIncrement,
                }),
                Statement::SingleExpression(Expression::SingleOperation {
                    base: Box::new(Expression::Identifier {
                        ident: Identifier(SpanData {
                            span: Span::new_source(source.clone(), 18..21),
                            data: "tmp".to_string(),
                        }),
                    }),
                    operation: SingleOperation::PrefixIncrement,
                }),
                Statement::SingleExpression(Expression::SingleOperation {
                    base: Box::new(Expression::Identifier {
                        ident: Identifier(SpanData {
                            span: Span::new_source(source.clone(), 27..30),
                            data: "tmp".to_string(),
                        }),
                    }),
                    operation: SingleOperation::SuffixDecrement,
                }),
                Statement::SingleExpression(Expression::SingleOperation {
                    base: Box::new(Expression::Identifier {
                        ident: Identifier(SpanData {
                            span: Span::new_source(source, 40..43),
                            data: "tmp".to_string(),
                        }),
                    }),
                    operation: SingleOperation::PrefixDecrement,
                }),
            ],
        },
    });

    let result = syntax::parse(input_tokenized);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn arrays() {
    let content = "
    tmp[0]++;
    ++tmp[0];
    tmp[0]--;
    --tmp[0];
        ";

    let source = Source::new("test", content);
    let input_span: Span = source.clone().into();
    let input_tokenized = tokenizer::tokenize(input_span);

    let expected = Ok(AST {
        global_scope: syntax::Scope {
            statements: vec![
                Statement::SingleExpression(Expression::SingleOperation {
                    base: Box::new(Expression::SingleOperation {
                        base: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 5..8),
                                data: "tmp".to_string(),
                            }),
                        }),
                        operation: SingleOperation::ArrayAccess(Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(source.clone(), 9..10),
                                data: "0".to_string(),
                            },
                        })),
                    }),
                    operation: SingleOperation::SuffixIncrement,
                }),
                Statement::SingleExpression(Expression::SingleOperation {
                    base: Box::new(Expression::SingleOperation {
                        base: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 21..24),
                                data: "tmp".to_string(),
                            }),
                        }),
                        operation: SingleOperation::ArrayAccess(Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(source.clone(), 25..26),
                                data: "0".to_string(),
                            },
                        })),
                    }),
                    operation: SingleOperation::PrefixIncrement,
                }),
                Statement::SingleExpression(Expression::SingleOperation {
                    base: Box::new(Expression::SingleOperation {
                        base: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 33..36),
                                data: "tmp".to_string(),
                            }),
                        }),
                        operation: SingleOperation::ArrayAccess(Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(source.clone(), 37..38),
                                data: "0".to_string(),
                            },
                        })),
                    }),
                    operation: SingleOperation::SuffixDecrement,
                }),
                Statement::SingleExpression(Expression::SingleOperation {
                    base: Box::new(Expression::SingleOperation {
                        base: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 49..52),
                                data: "tmp".to_string(),
                            }),
                        }),
                        operation: SingleOperation::ArrayAccess(Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(source, 53..54),
                                data: "0".to_string(),
                            },
                        })),
                    }),
                    operation: SingleOperation::PrefixDecrement,
                }),
            ],
        },
    });

    let result = syntax::parse(input_tokenized);
    dbg!(&result);

    assert_eq!(expected, result);
}
