use general::{Span, SpanData};
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
    let input_span = Span::from_parts("test", input, 0..input.len());
    let input_tokenized = tokenizer::tokenize(input_span);

    let expected = AST {
        global_scope: Scope {
            statements: vec![
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::from_parts("test", "int", 5..8),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::from_parts("test", "x", 9..10),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::from_parts("test", "a", 13..14),
                                data: "a".to_string(),
                            }),
                        }),
                        operation: ExpressionOperator::Add,
                        right: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::from_parts("test", "b", 17..18),
                                data: "b".to_string(),
                            }),
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::from_parts("test", "int", 24..27),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::from_parts("test", "x", 28..29),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::from_parts("test", "1", 32..33),
                                data: "1".to_string(),
                            },
                        }),
                        operation: ExpressionOperator::Sub,
                        right: Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::from_parts("test", "2", 36..37),
                                data: "2".to_string(),
                            },
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::from_parts("test", "int", 43..46),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::from_parts("test", "x", 47..48),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::from_parts("test", "a", 51..52),
                                data: "a".to_string(),
                            }),
                        }),
                        operation: ExpressionOperator::Multiply,
                        right: Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::from_parts("test", "2", 55..56),
                                data: "2".to_string(),
                            },
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::from_parts("test", "int", 62..65),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::from_parts("test", "x", 66..67),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Literal {
                            content: SpanData {
                                span: Span::from_parts("test", "1", 70..71),
                                data: "1".to_string(),
                            },
                        }),
                        operation: ExpressionOperator::Divide,
                        right: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::from_parts("test", "b", 74..75),
                                data: "b".to_string(),
                            }),
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::from_parts("test", "int", 81..84),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::from_parts("test", "x", 85..86),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::from_parts("test", "a", 89..90),
                                data: "a".to_string(),
                            }),
                        }),
                        operation: ExpressionOperator::Modulo,
                        right: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::from_parts("test", "b", 93..94),
                                data: "b".to_string(),
                            }),
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::from_parts("test", "int", 100..103),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::from_parts("test", "x", 104..105),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::SingleOperation {
                            operation: SingleOperation::Negative,
                            base: Box::new(Expression::Identifier {
                                ident: Identifier(SpanData {
                                    span: Span::from_parts("test", "a", 109..110),
                                    data: "a".to_string(),
                                }),
                            }),
                        }),
                        operation: ExpressionOperator::Add,
                        right: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::from_parts("test", "b", 113..114),
                                data: "b".to_string(),
                            }),
                        }),
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::from_parts("test", "int", 120..123),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::from_parts("test", "x", 124..125),
                        data: "x".to_string(),
                    }),
                    value: Expression::Operation {
                        left: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::from_parts("test", "a", 128..129),
                                data: "a".to_string(),
                            }),
                        }),
                        operation: ExpressionOperator::Add,
                        right: Box::new(Expression::SingleOperation {
                            operation: SingleOperation::Negative,
                            base: Box::new(Expression::Identifier {
                                ident: Identifier(SpanData {
                                    span: Span::from_parts("test", "b", 133..134),
                                    data: "b".to_string(),
                                }),
                            }),
                        }),
                    },
                },
            ],
        },
    };

    let result = parse(input_tokenized);

    dbg!(&expected);

    dbg!(&result);

    assert_eq!(expected, result);
}
