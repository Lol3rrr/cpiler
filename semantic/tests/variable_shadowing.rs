use general::{Source, Span, SpanData};
use semantic::{
    AAssignTarget, AExpression, APrimitive, AScope, AStatement, AType, FunctionDeclaration,
    Literal, AAST,
};
use syntax::Identifier;

#[test]
fn shadow_in_condition() {
    let content = "
int main() {
    int x = 0;

    if (1) {
        int x = 13;

        return x;
    }

    return x;
}
        ";
    let input_source = Source::new("test", content);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(input_tokens).unwrap();

    let expected = Ok(AAST {
        global_scope: semantic::ARootScope(AScope {
            statements: vec![],
            function_definitions: vec![(
                "main".to_string(),
                (
                    FunctionDeclaration {
                        arguments: vec![],
                        var_args: false,
                        return_ty: AType::Primitve(APrimitive::Int),
                        declaration: Span::new_source(input_source.clone(), 5..9),
                    },
                    AScope {
                        function_definitions: vec![].into_iter().collect(),
                        statements: vec![
                            AStatement::Assignment {
                                target: AAssignTarget::Variable {
                                    name: "x_15892656085374368478".to_string(),
                                    src: Identifier(SpanData {
                                        span: Span::new_source(input_source.clone(), 22..23),
                                        data: "x".to_string(),
                                    }),
                                    ty_info: SpanData {
                                        span: Span::new_source(input_source.clone(), 22..23),
                                        data: AType::Primitve(APrimitive::Int),
                                    },
                                },
                                value: AExpression::Cast {
                                    base: Box::new(AExpression::Literal(Literal::Integer(
                                        SpanData {
                                            span: Span::new_source(input_source.clone(), 26..27),
                                            data: 0,
                                        },
                                    ))),
                                    target: AType::Primitve(APrimitive::Int),
                                },
                            },
                            AStatement::If {
                                condition: AExpression::Literal(Literal::Integer(SpanData {
                                    span: Span::new_source(input_source.clone(), 38..39),
                                    data: 1,
                                })),
                                body: AScope {
                                    function_definitions: vec![].into_iter().collect(),
                                    statements: vec![
                                        AStatement::Assignment {
                                            target: AAssignTarget::Variable {
                                                name: "x_14047628523148287746".to_string(),
                                                src: Identifier(SpanData {
                                                    span: Span::new_source(
                                                        input_source.clone(),
                                                        55..56,
                                                    ),
                                                    data: "x".to_string(),
                                                }),
                                                ty_info: SpanData {
                                                    span: Span::new_source(
                                                        input_source.clone(),
                                                        55..56,
                                                    ),
                                                    data: AType::Primitve(APrimitive::Int),
                                                },
                                            },
                                            value: AExpression::Cast {
                                                base: Box::new(AExpression::Literal(
                                                    Literal::Integer(SpanData {
                                                        span: Span::new_source(
                                                            input_source.clone(),
                                                            59..61,
                                                        ),
                                                        data: 13,
                                                    }),
                                                )),
                                                target: AType::Primitve(APrimitive::Int),
                                            },
                                        },
                                        AStatement::Return {
                                            value: Some(AExpression::Variable {
                                                name: "x_14047628523148287746".to_string(),
                                                src: Identifier(SpanData {
                                                    span: Span::new_source(
                                                        input_source.clone(),
                                                        79..80,
                                                    ),
                                                    data: "x".to_string(),
                                                }),
                                                ty: SpanData {
                                                    span: Span::new_source(
                                                        input_source.clone(),
                                                        55..56,
                                                    ),
                                                    data: AType::Primitve(APrimitive::Int),
                                                },
                                            }),
                                        },
                                    ],
                                },
                                else_: None,
                            },
                            AStatement::Return {
                                value: Some(AExpression::Variable {
                                    name: "x_15892656085374368478".to_string(),
                                    src: Identifier(SpanData {
                                        span: Span::new_source(input_source.clone(), 100..101),
                                        data: "x".to_string(),
                                    }),
                                    ty: SpanData {
                                        span: Span::new_source(input_source.clone(), 22..23),
                                        data: AType::Primitve(APrimitive::Int),
                                    },
                                }),
                            },
                        ],
                    },
                ),
            )]
            .into_iter()
            .collect(),
        }),
    });

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert_eq!(expected, result);
}
