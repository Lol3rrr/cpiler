use std::collections::HashMap;

use general::{Source, Span, SpanData};
use semantic::{
    AAssignTarget, AExpression, AFunctionArg, AOperator, APrimitive, ARootScope, AScope,
    AStatement, AType, ArithemticOp, FunctionDeclaration, AAST,
};
use syntax::Identifier;

#[test]
fn function_def_args() {
    let input = "
void test(int arg_1, int arg_2) {
    int rand = arg_1 + arg_2;
}
        ";

    let input_source = Source::new("test", input);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(input_tokens).unwrap();

    let expected = Ok(AAST {
        global_scope: ARootScope(AScope {
            statements: vec![],
            function_definitions: vec![(
                "test".to_string(),
                (
                    FunctionDeclaration {
                        var_args: false,
                        return_ty: AType::Primitve(APrimitive::Void),
                        arguments: vec![
                            SpanData {
                                span: Span::new_source(input_source.clone(), 11..20),
                                data: AFunctionArg {
                                    name: Identifier(SpanData {
                                        data: "arg_1".to_string(),
                                        span: Span::new_source(input_source.clone(), 15..20),
                                    }),
                                    ty: AType::Primitve(APrimitive::Int),
                                },
                            },
                            SpanData {
                                span: Span::new_source(input_source.clone(), 22..31),
                                data: AFunctionArg {
                                    name: Identifier(SpanData {
                                        data: "arg_2".to_string(),
                                        span: Span::new_source(input_source.clone(), 26..31),
                                    }),
                                    ty: AType::Primitve(APrimitive::Int),
                                },
                            },
                        ],
                        declaration: Span::new_source(input_source.clone(), 6..10),
                    },
                    AScope {
                        function_definitions: HashMap::new(),
                        statements: vec![AStatement::Assignment {
                            target: AAssignTarget::Variable {
                                ident: Identifier(SpanData {
                                    span: Span::new_source(input_source.clone(), 43..47),
                                    data: "rand".to_string(),
                                }),
                                ty_info: SpanData {
                                    span: Span::new_source(input_source.clone(), 43..47),
                                    data: AType::Primitve(APrimitive::Int),
                                },
                            },
                            value: AExpression::BinaryOperator {
                                op: AOperator::Arithmetic(ArithemticOp::Add),
                                left: Box::new(AExpression::Variable {
                                    ident: Identifier(SpanData {
                                        span: Span::new_source(input_source.clone(), 50..55),
                                        data: "arg_1".to_string(),
                                    }),
                                    ty: AType::Primitve(APrimitive::Int),
                                }),
                                right: Box::new(AExpression::Variable {
                                    ident: Identifier(SpanData {
                                        span: Span::new_source(input_source.clone(), 58..63),
                                        data: "arg_2".to_string(),
                                    }),
                                    ty: AType::Primitve(APrimitive::Int),
                                }),
                            },
                        }],
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
