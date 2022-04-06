use general::{Source, Span, SpanData};
use semantic::{
    AAssignTarget, AExpression, APrimitive, ARootScope, AScope, AStatement, AType,
    FunctionDeclaration, Literal, SemanticError, AAST,
};
use syntax::Identifier;

#[test]
fn valid_while_loop() {
    let input_content = "
void other() {
    int test;
    while(1) {
        test = 1;
    }
}
        ";
    let input_source = Source::new("test", input_content);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(input_tokens).unwrap();

    let expected = Ok(AAST {
        global_scope: ARootScope(AScope {
            statements: vec![],
            function_definitions: vec![(
                "other".to_string(),
                (
                    FunctionDeclaration {
                        return_ty: AType::Primitve(APrimitive::Void),
                        arguments: vec![],
                        declaration: Span::new_source(input_source.clone(), 6..11),
                        var_args: false,
                    },
                    AScope {
                        statements: vec![
                            AStatement::DeclareVar {
                                name: "test_2149230751987271372".to_string(),
                                src: Identifier(SpanData {
                                    span: Span::new_source(input_source.clone(), 24..28),
                                    data: "test".to_string(),
                                }),
                                ty: AType::Primitve(APrimitive::Int),
                            },
                            AStatement::WhileLoop {
                                condition: AExpression::Literal(Literal::Integer(SpanData {
                                    span: Span::new_source(input_source.clone(), 40..41),
                                    data: 1,
                                })),
                                body: AScope {
                                    statements: vec![AStatement::Assignment {
                                        target: AAssignTarget::Variable {
                                            name: "test_2149230751987271372".to_string(),
                                            src: Identifier(SpanData {
                                                span: Span::new_source(
                                                    input_source.clone(),
                                                    53..57,
                                                ),
                                                data: "test".to_string(),
                                            }),
                                            ty_info: SpanData {
                                                span: Span::new_source(
                                                    input_source.clone(),
                                                    24..28,
                                                ),
                                                data: AType::Primitve(APrimitive::Int),
                                            },
                                        },
                                        value: AExpression::Cast {
                                            base: Box::new(AExpression::Literal(Literal::Integer(
                                                SpanData {
                                                    span: Span::new_source(input_source, 60..61),
                                                    data: 1,
                                                },
                                            ))),
                                            target: AType::Primitve(APrimitive::Int),
                                        },
                                    }],
                                    function_definitions: vec![].into_iter().collect(),
                                },
                            },
                        ],
                        function_definitions: vec![].into_iter().collect(),
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

#[ignore = "This detail must be reviewed later on"]
#[test]
fn wrong_condition_type() {
    let input_content = "
void other() {
    int test;
    while(1.0f) {
        test = 1;
    }
}
        ";
    let input_source = Source::new("test", input_content);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(input_tokens).unwrap();

    let expected = Err(SemanticError::MismatchedTypes {
        expected: SpanData {
            span: Span::new_source(input_source.clone(), 0..1),
            data: AType::Primitve(APrimitive::Int),
        },
        received: SpanData {
            span: Span::new_source(input_source, 0..1),
            data: AType::Primitve(APrimitive::Float),
        },
    });

    let result = semantic::parse(input_ast);

    dbg!(&result);

    assert_eq!(expected, result);
}
