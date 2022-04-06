use general::{Source, Span, SpanData};
use semantic::{
    AAssignTarget, AExpression, APrimitive, ARootScope, AScope, AStatement, AType, Literal, AAST,
};
use syntax::Identifier;

#[test]
fn basic_only_output() {
    let input = "
int out;
asm(\"mov ${out}, 13\", out);
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let input_ast = syntax::parse(tokens).unwrap();

    let expected = Ok(AAST {
        global_scope: ARootScope(AScope {
            function_definitions: vec![].into_iter().collect(),
            statements: vec![
                AStatement::DeclareVar {
                    name: "out_3599231011511905905".to_string(),
                    src: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 5..8),
                        data: "out".to_string(),
                    }),
                    ty: AType::Primitve(APrimitive::Int),
                },
                AStatement::Expression(AExpression::InlineAssembly {
                    span: Span::new_source(source.clone(), 10..13),
                    template: SpanData {
                        span: Span::new_source(source.clone(), 14..30),
                        data: "mov ${out}, 13".to_string(),
                    },
                    input_vars: vec![],
                    output_var: Some((
                        Identifier(SpanData {
                            span: Span::new_source(source.clone(), 32..35),
                            data: "out".to_string(),
                        }),
                        SpanData {
                            span: Span::new_source(source, 5..8),
                            data: AType::Primitve(APrimitive::Int),
                        },
                    )),
                }),
            ],
        }),
    });

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn basic_input_output() {
    let input = "
int out;
int in = 13;
asm(\"mov ${out}, ${in}\", out, in);
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let input_ast = syntax::parse(tokens).unwrap();

    let expected = Ok(AAST {
        global_scope: ARootScope(AScope {
            function_definitions: vec![].into_iter().collect(),
            statements: vec![
                AStatement::DeclareVar {
                    name: "out_3599231011511905905".to_string(),
                    src: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 5..8),
                        data: "out".to_string(),
                    }),
                    ty: AType::Primitve(APrimitive::Int),
                },
                AStatement::Assignment {
                    target: AAssignTarget::Variable {
                        name: "in_9219883154635435264".to_string(),
                        src: Identifier(SpanData {
                            span: Span::new_source(source.clone(), 14..16),
                            data: "in".to_string(),
                        }),
                        ty_info: SpanData {
                            span: Span::new_source(source.clone(), 14..16),
                            data: AType::Primitve(APrimitive::Int),
                        },
                    },
                    value: AExpression::Cast {
                        base: Box::new(AExpression::Literal(Literal::Integer(SpanData {
                            span: Span::new_source(source.clone(), 19..21),
                            data: 13,
                        }))),
                        target: AType::Primitve(APrimitive::Int),
                    },
                },
                AStatement::Expression(AExpression::InlineAssembly {
                    span: Span::new_source(source.clone(), 23..26),
                    template: SpanData {
                        span: Span::new_source(source.clone(), 27..46),
                        data: "mov ${out}, ${in}".to_string(),
                    },
                    input_vars: vec![(
                        Identifier(SpanData {
                            span: Span::new_source(source.clone(), 53..55),
                            data: "in".to_string(),
                        }),
                        SpanData {
                            span: Span::new_source(source.clone(), 14..16),
                            data: AType::Primitve(APrimitive::Int),
                        },
                    )],
                    output_var: Some((
                        Identifier(SpanData {
                            span: Span::new_source(source.clone(), 48..51),
                            data: "out".to_string(),
                        }),
                        SpanData {
                            span: Span::new_source(source, 5..8),
                            data: AType::Primitve(APrimitive::Int),
                        },
                    )),
                }),
            ],
        }),
    });

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert_eq!(expected, result);
}
