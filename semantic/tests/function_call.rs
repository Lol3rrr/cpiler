use general::{Source, Span, SpanData};
use semantic::{
    AExpression, APrimitive, ARootScope, AScope, AStatement, AType, FunctionDeclaration, Literal,
    SemanticError, AAST,
};
use syntax::Identifier;

#[test]
fn mismatched_arg_count() {
    let input_content = "
void testing(int test);

void other() {
    testing();
}
        ";
    let input_source = Source::new("test", input_content);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(input_tokens).unwrap();

    let expected = Err(SemanticError::MismatchedFunctionArgsCount {
        expected: SpanData {
            span: Span::new_source(input_source.clone(), 6..13),
            data: 1,
        },
        received: SpanData {
            span: Span::new_source(input_source.clone(), 45..52),
            data: 0,
        },
    });

    let result = semantic::parse(input_ast);

    assert_eq!(expected, result);
}

#[test]
#[ignore = "Im not quite sure what we expect here to happen as I would prefer an error but other compilers let it slip"]
fn mismatched_arg_types() {
    let input_content = "
void testing(int test);

void other() {
    testing(4.0f);
}
        ";

    let input_source = Source::new("test", input_content);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(input_tokens).unwrap();

    let expected = Err(SemanticError::MismatchedTypes {
        expected: SpanData {
            span: Span::new_source(input_source.clone(), 14..22),
            data: AType::Primitve(APrimitive::Int),
        },
        received: SpanData {
            span: Span::new_source(input_source.clone(), 53..56),
            data: AType::Primitve(APrimitive::Float),
        },
    });

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn valid() {
    let input_content = "
void testing(int test);

void other() {
    testing(13);
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
                        arguments: vec![],
                        declaration: Span::new_source(input_source.clone(), 31..36),
                        return_ty: AType::Primitve(APrimitive::Void),
                        var_args: false,
                    },
                    AScope {
                        statements: vec![AStatement::Expression(AExpression::FunctionCall {
                            name: Identifier(SpanData {
                                span: Span::new_source(input_source.clone(), 45..52),
                                data: "testing".to_string(),
                            }),
                            arguments: vec![AExpression::Literal(Literal::Integer(SpanData {
                                span: Span::new_source(input_source.clone(), 53..55),
                                data: 13,
                            }))],
                            result_ty: AType::Primitve(APrimitive::Void),
                        })],
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
