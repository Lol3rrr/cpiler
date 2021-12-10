use general::{Source, Span, SpanData};
use syntax::{DataType, Expression, Identifier, Scope, Statement, TypeToken, AST};

#[test]
fn simple_if() {
    let input = "
if (13) {
    int x;
}
        ";
    let source: Source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::If {
                condition: Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 5..7),
                        data: "13".to_string(),
                    },
                },
                scope: Scope {
                    statements: vec![Statement::VariableDeclaration {
                        ty: TypeToken::Primitive(SpanData {
                            span: Span::new_source(source.clone(), 15..18),
                            data: DataType::Int,
                        }),
                        name: Identifier(SpanData {
                            span: Span::new_source(source.clone(), 19..20),
                            data: "x".to_string(),
                        }),
                    }],
                },
                elses: None,
            }],
        },
    });

    let result = syntax::parse(tokens);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn if_else() {
    let input = "
if (13) {
    int x;
} else {
    int y;
}
        ";
    let source: Source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::If {
                condition: Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 5..7),
                        data: "13".to_string(),
                    },
                },
                scope: Scope {
                    statements: vec![Statement::VariableDeclaration {
                        ty: TypeToken::Primitive(SpanData {
                            span: Span::new_source(source.clone(), 15..18),
                            data: DataType::Int,
                        }),
                        name: Identifier(SpanData {
                            span: Span::new_source(source.clone(), 19..20),
                            data: "x".to_string(),
                        }),
                    }],
                },
                elses: Some(Scope {
                    statements: vec![Statement::VariableDeclaration {
                        ty: TypeToken::Primitive(SpanData {
                            span: Span::new_source(source.clone(), 35..38),
                            data: DataType::Int,
                        }),
                        name: Identifier(SpanData {
                            span: Span::new_source(source.clone(), 39..40),
                            data: "y".to_string(),
                        }),
                    }],
                }),
            }],
        },
    });

    let result = syntax::parse(tokens);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn if_elseif_else() {
    let input = "
if (13) {
    int x;
} else if(23) {
    int y;
} else {
    int z;
}
        ";
    let source: Source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::If {
                condition: Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 5..7),
                        data: "13".to_string(),
                    },
                },
                scope: Scope {
                    statements: vec![Statement::VariableDeclaration {
                        ty: TypeToken::Primitive(SpanData {
                            span: Span::new_source(source.clone(), 15..18),
                            data: DataType::Int,
                        }),
                        name: Identifier(SpanData {
                            span: Span::new_source(source.clone(), 19..20),
                            data: "x".to_string(),
                        }),
                    }],
                },
                elses: Some(Scope {
                    statements: vec![Statement::If {
                        condition: Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(source.clone(), 32..34),
                                data: "23".to_string(),
                            },
                        },
                        scope: Scope {
                            statements: vec![Statement::VariableDeclaration {
                                ty: TypeToken::Primitive(SpanData {
                                    span: Span::new_source(source.clone(), 42..45),
                                    data: DataType::Int,
                                }),
                                name: Identifier(SpanData {
                                    span: Span::new_source(source.clone(), 46..47),
                                    data: "y".to_string(),
                                }),
                            }],
                        },
                        elses: Some(Scope {
                            statements: vec![Statement::VariableDeclaration {
                                ty: TypeToken::Primitive(SpanData {
                                    span: Span::new_source(source.clone(), 62..65),
                                    data: DataType::Int,
                                }),
                                name: Identifier(SpanData {
                                    span: Span::new_source(source.clone(), 66..67),
                                    data: "z".to_string(),
                                }),
                            }],
                        }),
                    }],
                }),
            }],
        },
    });

    let result = syntax::parse(tokens);
    dbg!(&result);

    assert_eq!(expected, result);
}
