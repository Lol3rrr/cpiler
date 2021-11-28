use general::{Source, Span, SpanData};
use syntax::{
    DataType, FunctionArgument, FunctionHead, Identifier, Scope, Statement, TypeToken, AST,
};

#[test]
fn function_declarations() {
    let input_content = "void src();";
    let input_source = Source::new("test", input_content);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::FunctionDeclaration(FunctionHead {
                name: Identifier(SpanData {
                    span: Span::new_source(input_source.clone(), 5..8),
                    data: "src".to_string(),
                }),
                r_type: TypeToken::Primitive(SpanData {
                    span: Span::new_source(input_source.clone(), 0..4),
                    data: DataType::Void,
                }),
                arguments: vec![],
                var_args: false,
            })],
        },
    });

    let result = syntax::parse(input_tokens);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn function_declaration_pointer_arg() {
    let input_content = "void src(int *first);";
    let input_source = Source::new("test", input_content);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::FunctionDeclaration(FunctionHead {
                name: Identifier(SpanData {
                    span: Span::new_source(input_source.clone(), 5..8),
                    data: "src".to_string(),
                }),
                r_type: TypeToken::Primitive(SpanData {
                    span: Span::new_source(input_source.clone(), 0..4),
                    data: DataType::Void,
                }),
                arguments: vec![SpanData {
                    span: Span::new_source(input_source.clone(), 9..19),
                    data: FunctionArgument {
                        name: Identifier(SpanData {
                            span: Span::new_source(input_source.clone(), 14..19),
                            data: "first".to_string(),
                        }),
                        ty: TypeToken::Pointer(Box::new(TypeToken::Primitive(SpanData {
                            span: Span::new_source(input_source.clone(), 9..12),
                            data: DataType::Int,
                        }))),
                    },
                }],
                var_args: false,
            })],
        },
    });

    let result = syntax::parse(input_tokens);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn function_declaration_two_args() {
    let input_content = "void src(int *first, int second);";
    let input_source = Source::new("test", input_content);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::FunctionDeclaration(FunctionHead {
                name: Identifier(SpanData {
                    span: Span::new_source(input_source.clone(), 5..8),
                    data: "src".to_string(),
                }),
                r_type: TypeToken::Primitive(SpanData {
                    span: Span::new_source(input_source.clone(), 0..4),
                    data: DataType::Void,
                }),
                arguments: vec![
                    SpanData {
                        span: Span::new_source(input_source.clone(), 9..19),
                        data: FunctionArgument {
                            name: Identifier(SpanData {
                                span: Span::new_source(input_source.clone(), 14..19),
                                data: "first".to_string(),
                            }),
                            ty: TypeToken::Pointer(Box::new(TypeToken::Primitive(SpanData {
                                span: Span::new_source(input_source.clone(), 9..12),
                                data: DataType::Int,
                            }))),
                        },
                    },
                    SpanData {
                        span: Span::new_source(input_source.clone(), 21..31),
                        data: FunctionArgument {
                            name: Identifier(SpanData {
                                span: Span::new_source(input_source.clone(), 25..31),
                                data: "second".to_string(),
                            }),
                            ty: TypeToken::Primitive(SpanData {
                                span: Span::new_source(input_source.clone(), 21..24),
                                data: DataType::Int,
                            }),
                        },
                    },
                ],
                var_args: false,
            })],
        },
    });

    let result = syntax::parse(input_tokens);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn function_declaration_with_var_args() {
    let input_content = "void src(int *first, ...);";
    let input_source = Source::new("test", input_content);
    let input_span: Span = input_source.clone().into();
    let input_tokens = tokenizer::tokenize(input_span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::FunctionDeclaration(FunctionHead {
                name: Identifier(SpanData {
                    span: Span::new_source(input_source.clone(), 5..8),
                    data: "src".to_string(),
                }),
                r_type: TypeToken::Primitive(SpanData {
                    span: Span::new_source(input_source.clone(), 0..4),
                    data: DataType::Void,
                }),
                arguments: vec![SpanData {
                    span: Span::new_source(input_source.clone(), 9..19),
                    data: FunctionArgument {
                        name: Identifier(SpanData {
                            span: Span::new_source(input_source.clone(), 14..19),
                            data: "first".to_string(),
                        }),
                        ty: TypeToken::Pointer(Box::new(TypeToken::Primitive(SpanData {
                            span: Span::new_source(input_source.clone(), 9..12),
                            data: DataType::Int,
                        }))),
                    },
                }],
                var_args: true,
            })],
        },
    });

    let result = syntax::parse(input_tokens);
    dbg!(&result);

    assert_eq!(expected, result);
}
