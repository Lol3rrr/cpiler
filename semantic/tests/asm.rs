use general::{Source, Span, SpanData};
use semantic::{
    AAssignTarget, AExpression, APrimitive, ARootScope, AScope, AStatement, AType, Literal, AAST,
};
use syntax::Identifier;

#[test]
fn basic_only_output() {
    let input = "
int out;
asm(\"mov ${out}, 13\", {}, out);
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let input_ast = syntax::parse(tokens).unwrap();

    let expected = Ok(AAST {
        global_scope: ARootScope(AScope {
            function_definitions: vec![].into_iter().collect(),
            statements: vec![AStatement::Expression(AExpression::InlineAssembly {
                span: Span::new_source(source.clone(), 10..13),
                template: SpanData {
                    span: Span::new_source(source.clone(), 14..30),
                    data: "mov ${out}, 13".to_string(),
                },
                input_vars: vec![],
                output_var: (
                    Identifier(SpanData {
                        span: Span::new_source(source.clone(), 36..39),
                        data: "out".to_string(),
                    }),
                    SpanData {
                        span: Span::new_source(source.clone(), 5..8),
                        data: AType::Primitve(APrimitive::Int),
                    },
                ),
            })],
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
asm(\"mov ${out}, ${in}\", {in}, out);
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let input_ast = syntax::parse(tokens).unwrap();

    let expected = Ok(AAST {
        global_scope: ARootScope(AScope {
            function_definitions: vec![].into_iter().collect(),
            statements: vec![
                AStatement::Assignment {
                    target: AAssignTarget::Variable {
                        ident: Identifier(SpanData {
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
                            span: Span::new_source(source.clone(), 49..51),
                            data: "in".to_string(),
                        }),
                        SpanData {
                            span: Span::new_source(source.clone(), 14..16),
                            data: AType::Primitve(APrimitive::Int),
                        },
                    )],
                    output_var: (
                        Identifier(SpanData {
                            span: Span::new_source(source.clone(), 54..57),
                            data: "out".to_string(),
                        }),
                        SpanData {
                            span: Span::new_source(source.clone(), 5..8),
                            data: AType::Primitve(APrimitive::Int),
                        },
                    ),
                }),
            ],
        }),
    });

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert_eq!(expected, result);
}
