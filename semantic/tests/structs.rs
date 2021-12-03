use general::{Source, Span, SpanData};
use semantic::{
    AAssignTarget, AExpression, APrimitive, ARootScope, AScope, AStatement, AType, Literal,
    StructDef, StructMember, AAST,
};
use syntax::Identifier;

#[test]
fn assign_field() {
    let content = "
struct tmp {
    int first;
};

struct tmp var;
var.first = 1;
        ";
    let source = Source::new("test", content);
    let input_span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(tokens).unwrap();

    let expected = Ok(AAST {
        global_scope: ARootScope(AScope {
            statements: vec![AStatement::Assignment {
                target: AAssignTarget::StructField {
                    target: Box::new(AAssignTarget::Variable {
                        ident: Identifier(SpanData {
                            span: Span::new_source(source.clone(), 49..52),
                            data: "var".to_string(),
                        }),
                        ty_info: SpanData {
                            span: Span::new_source(source.clone(), 44..47),
                            data: AType::Struct(StructDef {
                                members: vec![StructMember {
                                    name: Identifier(SpanData {
                                        span: Span::new_source(source.clone(), 22..27),
                                        data: "first".to_string(),
                                    }),
                                    ty: AType::Primitve(APrimitive::Int),
                                }],
                            }),
                        },
                    }),
                    field: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 53..58),
                        data: "first".to_string(),
                    }),
                    ty_info: SpanData {
                        span: Span::new_source(source.clone(), 22..27),
                        data: AType::Primitve(APrimitive::Int),
                    },
                },
                value: AExpression::Literal(Literal::Integer(SpanData {
                    span: Span::new_source(source.clone(), 61..62),
                    data: 1,
                })),
            }],
            function_definitions: vec![].into_iter().collect(),
        }),
    });

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert_eq!(expected, result);
}
