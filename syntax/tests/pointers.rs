use general::{Source, Span, SpanData};
use syntax::{DataType, Expression, Identifier, Scope, SingleOperation, Statement, TypeToken, AST};

#[test]
fn address_of_var() {
    let input = "
int* y = &x;
        ";
    let source: Source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::VariableDeclarationAssignment {
                name: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 6..7),
                    data: "y".to_string(),
                }),
                ty: TypeToken::Pointer(Box::new(TypeToken::Primitive(SpanData {
                    span: Span::new_source(source.clone(), 1..4),
                    data: DataType::Int,
                }))),
                value: Expression::SingleOperation {
                    operation: SingleOperation::AddressOf,
                    base: Box::new(Expression::Identifier {
                        ident: Identifier(SpanData {
                            span: Span::new_source(source.clone(), 11..12),
                            data: "x".to_string(),
                        }),
                    }),
                },
            }],
        },
    });

    let result = syntax::parse(tokens);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn write_to_ptr() {
    let input = "
*y = 13;
        ";
    let source: Source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::VariableDerefAssignment {
                target: Expression::Identifier {
                    ident: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 2..3),
                        data: "y".to_string(),
                    }),
                },
                value: Expression::Literal {
                    content: SpanData {
                        span: Span::new_source(source.clone(), 6..8),
                        data: "13".to_string(),
                    },
                },
            }],
        },
    });

    let result = syntax::parse(tokens);
    dbg!(&result);

    assert_eq!(expected, result);
}
