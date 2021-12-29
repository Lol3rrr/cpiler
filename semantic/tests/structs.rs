use general::{Source, Span, SpanData};
use semantic::{
    AAssignTarget, AExpression, APrimitive, ARootScope, AScope, AStatement, AType, Literal,
    SemanticError, StructDef, StructFieldTarget, StructMember, AAST,
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

    let struct_def = StructDef {
        members: vec![StructMember {
            name: Identifier(SpanData {
                span: Span::new_source(source.clone(), 22..27),
                data: "first".to_string(),
            }),
            ty: AType::Primitve(APrimitive::Int),
        }],
    };

    let expected = Ok(AAST {
        global_scope: ARootScope(AScope {
            statements: vec![
                AStatement::DeclareVar {
                    name: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 44..47),
                        data: "var".to_string(),
                    }),
                    ty: AType::Struct {
                        def: struct_def.clone(),
                        area: Span::new_source(source.clone(), 12..31),
                    },
                },
                AStatement::Assignment {
                    target: AAssignTarget::StructField(StructFieldTarget {
                        target: Box::new(AAssignTarget::Variable {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 49..52),
                                data: "var".to_string(),
                            }),
                            ty_info: SpanData {
                                span: Span::new_source(source.clone(), 44..47),
                                data: AType::Struct {
                                    def: struct_def.clone(),
                                    area: Span::new_source(source.clone(), 12..31),
                                },
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
                    }),
                    value: AExpression::Cast {
                        base: Box::new(AExpression::Literal(Literal::Integer(SpanData {
                            span: Span::new_source(source.clone(), 61..62),
                            data: 1,
                        }))),
                        target: AType::Primitve(APrimitive::Int),
                    },
                },
            ],
            function_definitions: vec![].into_iter().collect(),
        }),
    });

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn assign_unknown_field() {
    let content = "
struct tmp {
    int first;
};

struct tmp var;
var.second = 1;
        ";
    let source = Source::new("test", content);
    let input_span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(tokens).unwrap();

    let struct_def = StructDef {
        members: vec![StructMember {
            name: Identifier(SpanData {
                span: Span::new_source(source.clone(), 22..27),
                data: "first".to_string(),
            }),
            ty: AType::Primitve(APrimitive::Int),
        }],
    };

    let expected = Err(SemanticError::UnknownStructField {
        field_name: Identifier(SpanData {
            span: Span::new_source(source.clone(), 53..59),
            data: "second".to_string(),
        }),
        struct_def: SpanData {
            span: Span::new_source(source.clone(), 12..31),
            data: struct_def,
        },
    });

    let result = semantic::parse(input_ast);
    dbg!(&result);

    assert_eq!(expected, result);
}
