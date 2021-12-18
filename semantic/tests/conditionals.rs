use general::{Source, Span, SpanData};
use semantic::{
    AAssignTarget, AExpression, APrimitive, ARootScope, AScope, AStatement, AType,
    FunctionDeclaration, Literal, AAST,
};
use syntax::Identifier;

#[test]
fn single_if() {
    let content = "
void test() {
    int before = 0;
    if (13) {
        int inner = 13;
    }
    int after = 0;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();

    let expected = Ok(AAST {
        global_scope: ARootScope(AScope {
            statements: vec![],
            function_definitions: vec![(
                "test".to_string(),
                (
                    FunctionDeclaration {
                        var_args: false,
                        return_ty: AType::Primitve(APrimitive::Void),
                        arguments: vec![],
                        declaration: Span::new_source(source.clone(), 6..10),
                    },
                    AScope {
                        function_definitions: vec![].into_iter().collect(),
                        statements: vec![
                            AStatement::Assignment {
                                target: AAssignTarget::Variable {
                                    ident: Identifier(SpanData {
                                        span: Span::new_source(source.clone(), 23..29),
                                        data: "before".to_string(),
                                    }),
                                    ty_info: SpanData {
                                        span: Span::new_source(source.clone(), 23..29),
                                        data: AType::Primitve(APrimitive::Int),
                                    },
                                },
                                value: AExpression::Cast {
                                    target: AType::Primitve(APrimitive::Int),
                                    base: Box::new(AExpression::Literal(Literal::Integer(
                                        SpanData {
                                            span: Span::new_source(source.clone(), 32..33),
                                            data: 0,
                                        },
                                    ))),
                                },
                            },
                            AStatement::If {
                                condition: AExpression::Literal(Literal::Integer(SpanData {
                                    span: Span::new_source(source.clone(), 43..45),
                                    data: 13,
                                })),
                                body: AScope {
                                    function_definitions: vec![].into_iter().collect(),
                                    statements: vec![AStatement::Assignment {
                                        target: AAssignTarget::Variable {
                                            ident: Identifier(SpanData {
                                                span: Span::new_source(source.clone(), 61..66),
                                                data: "inner".to_string(),
                                            }),
                                            ty_info: SpanData {
                                                span: Span::new_source(source.clone(), 61..66),
                                                data: AType::Primitve(APrimitive::Int),
                                            },
                                        },
                                        value: AExpression::Cast {
                                            target: AType::Primitve(APrimitive::Int),
                                            base: Box::new(AExpression::Literal(Literal::Integer(
                                                SpanData {
                                                    span: Span::new_source(source.clone(), 69..71),
                                                    data: 13,
                                                },
                                            ))),
                                        },
                                    }],
                                },
                                else_: None,
                            },
                            AStatement::Assignment {
                                target: AAssignTarget::Variable {
                                    ident: Identifier(SpanData {
                                        span: Span::new_source(source.clone(), 87..92),
                                        data: "after".to_string(),
                                    }),
                                    ty_info: SpanData {
                                        span: Span::new_source(source.clone(), 87..92),
                                        data: AType::Primitve(APrimitive::Int),
                                    },
                                },
                                value: AExpression::Cast {
                                    target: AType::Primitve(APrimitive::Int),
                                    base: Box::new(AExpression::Literal(Literal::Integer(
                                        SpanData {
                                            span: Span::new_source(source.clone(), 95..96),
                                            data: 0,
                                        },
                                    ))),
                                },
                            },
                        ],
                    },
                ),
            )]
            .into_iter()
            .collect(),
        }),
    });

    let result = semantic::parse(syntax_ast);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn if_else() {
    let content = "
void test() {
    int before = 0;
    if (13) {
        int inner_1 = 13;
    } else {
        int inner_2 = 23;
    }
    int after = 0;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();

    let expected = Ok(AAST {
        global_scope: ARootScope(AScope {
            statements: vec![],
            function_definitions: vec![(
                "test".to_string(),
                (
                    FunctionDeclaration {
                        var_args: false,
                        return_ty: AType::Primitve(APrimitive::Void),
                        arguments: vec![],
                        declaration: Span::new_source(source.clone(), 6..10),
                    },
                    AScope {
                        function_definitions: vec![].into_iter().collect(),
                        statements: vec![
                            AStatement::Assignment {
                                target: AAssignTarget::Variable {
                                    ident: Identifier(SpanData {
                                        span: Span::new_source(source.clone(), 23..29),
                                        data: "before".to_string(),
                                    }),
                                    ty_info: SpanData {
                                        span: Span::new_source(source.clone(), 23..29),
                                        data: AType::Primitve(APrimitive::Int),
                                    },
                                },
                                value: AExpression::Cast {
                                    target: AType::Primitve(APrimitive::Int),
                                    base: Box::new(AExpression::Literal(Literal::Integer(
                                        SpanData {
                                            span: Span::new_source(source.clone(), 32..33),
                                            data: 0,
                                        },
                                    ))),
                                },
                            },
                            AStatement::If {
                                condition: AExpression::Literal(Literal::Integer(SpanData {
                                    span: Span::new_source(source.clone(), 43..45),
                                    data: 13,
                                })),
                                body: AScope {
                                    function_definitions: vec![].into_iter().collect(),
                                    statements: vec![AStatement::Assignment {
                                        target: AAssignTarget::Variable {
                                            ident: Identifier(SpanData {
                                                span: Span::new_source(source.clone(), 61..68),
                                                data: "inner_1".to_string(),
                                            }),
                                            ty_info: SpanData {
                                                span: Span::new_source(source.clone(), 61..68),
                                                data: AType::Primitve(APrimitive::Int),
                                            },
                                        },
                                        value: AExpression::Cast {
                                            target: AType::Primitve(APrimitive::Int),
                                            base: Box::new(AExpression::Literal(Literal::Integer(
                                                SpanData {
                                                    span: Span::new_source(source.clone(), 71..73),
                                                    data: 13,
                                                },
                                            ))),
                                        },
                                    }],
                                },
                                else_: Some(AScope {
                                    function_definitions: vec![].into_iter().collect(),
                                    statements: vec![AStatement::Assignment {
                                        target: AAssignTarget::Variable {
                                            ident: Identifier(SpanData {
                                                span: Span::new_source(source.clone(), 100..107),
                                                data: "inner_2".to_string(),
                                            }),
                                            ty_info: SpanData {
                                                span: Span::new_source(source.clone(), 100..107),
                                                data: AType::Primitve(APrimitive::Int),
                                            },
                                        },
                                        value: AExpression::Cast {
                                            target: AType::Primitve(APrimitive::Int),
                                            base: Box::new(AExpression::Literal(Literal::Integer(
                                                SpanData {
                                                    span: Span::new_source(
                                                        source.clone(),
                                                        110..112,
                                                    ),
                                                    data: 23,
                                                },
                                            ))),
                                        },
                                    }],
                                }),
                            },
                            AStatement::Assignment {
                                target: AAssignTarget::Variable {
                                    ident: Identifier(SpanData {
                                        span: Span::new_source(source.clone(), 128..133),
                                        data: "after".to_string(),
                                    }),
                                    ty_info: SpanData {
                                        span: Span::new_source(source.clone(), 128..133),
                                        data: AType::Primitve(APrimitive::Int),
                                    },
                                },
                                value: AExpression::Cast {
                                    target: AType::Primitve(APrimitive::Int),
                                    base: Box::new(AExpression::Literal(Literal::Integer(
                                        SpanData {
                                            span: Span::new_source(source.clone(), 136..137),
                                            data: 0,
                                        },
                                    ))),
                                },
                            },
                        ],
                    },
                ),
            )]
            .into_iter()
            .collect(),
        }),
    });

    let result = semantic::parse(syntax_ast);
    dbg!(&result);

    assert_eq!(expected, result);
}
