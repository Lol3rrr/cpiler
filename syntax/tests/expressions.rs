use general::{Source, Span, SpanData};
use syntax::{
    parse, Expression, ExpressionOperator, Identifier, Scope, SingleOperation, Statement,
    TypeToken, AST,
};
use tokenizer::DataType;

#[test]
fn arithemtic() {
    let input = "
    int x = a + b;
    int x = 1 - 2;
    int x = a * 2;
    int x = 1 / b;
    int x = a % b;
    int x = -a + b;
    int x = a + -b;
        ";

    let source = Source::new("test", input);
    let input_span: Span = source.clone().into();
    let input_tokenized = tokenizer::tokenize(input_span);

    let expected = AST {
        global_scope: Scope {
            statements: vec![
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 5..8),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 9..10),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 13..14),
                                data: "a".to_string(),
                            }),
                        }),
                        operation: ExpressionOperator::Add,
                        right: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 17..18),
                                data: "b".to_string(),
                            }),
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 24..27),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 28..29),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(source.clone(), 32..33),
                                data: "1".to_string(),
                            },
                        }),
                        operation: ExpressionOperator::Sub,
                        right: Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(source.clone(), 36..37),
                                data: "2".to_string(),
                            },
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 43..46),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 47..48),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 51..52),
                                data: "a".to_string(),
                            }),
                        }),
                        operation: ExpressionOperator::Multiply,
                        right: Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(source.clone(), 55..56),
                                data: "2".to_string(),
                            },
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 62..65),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 66..67),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::new_source(source.clone(), 70..71),
                                data: "1".to_string(),
                            },
                        }),
                        operation: ExpressionOperator::Divide,
                        right: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 74..75),
                                data: "b".to_string(),
                            }),
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 81..84),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 85..86),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 89..90),
                                data: "a".to_string(),
                            }),
                        }),
                        operation: ExpressionOperator::Modulo,
                        right: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 93..94),
                                data: "b".to_string(),
                            }),
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 100..103),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 104..105),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::SingleOperation {
                            operation: SingleOperation::Negative,
                            base: Box::new(Expression::Identifier {
                                ident: Identifier(SpanData {
                                    span: Span::new_source(source.clone(), 109..110),
                                    data: "a".to_string(),
                                }),
                            }),
                        }),
                        operation: ExpressionOperator::Add,
                        right: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 113..114),
                                data: "b".to_string(),
                            }),
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 120..123),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 124..125),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source.clone(), 128..129),
                                data: "a".to_string(),
                            }),
                        }),
                        operation: ExpressionOperator::Add,
                        right: Box::new(Expression::SingleOperation {
                            operation: SingleOperation::Negative,
                            base: Box::new(Expression::Identifier {
                                ident: Identifier(SpanData {
                                    span: Span::new_source(source.clone(), 133..134),
                                    data: "b".to_string(),
                                }),
                            }),
                        }),
                    },
                },
            ],
        },
    };

    let result = parse(input_tokenized).unwrap();

    dbg!(&expected);

    dbg!(&result);

    assert_eq!(expected, result);
}
