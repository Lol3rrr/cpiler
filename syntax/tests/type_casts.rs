use general::{Source, Span, SpanData};
use syntax::{Expression, Identifier, Scope, Statement, TypeToken, AST};
use tokenizer::DataType;

#[test]
fn primitive_cast() {
    let content = "
int first = 123;
char casted = (char) first;
        ";
    let source = Source::new("test", content);
    let input_span: Span = source.clone().into();
    let tokenized = tokenizer::tokenize(input_span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 1..4),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 5..10),
                        data: "first".to_string(),
                    }),
                    value: Expression::Literal {
                        content: SpanData {
                            span: Span::new_source(source.clone(), 13..16),
                            data: "123".to_string(),
                        },
                    },
                },
                Statement::VariableDeclarationAssignment {
                    ty: TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 18..22),
                        data: DataType::Char,
                    }),
                    name: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 23..29),
                        data: "casted".to_string(),
                    }),
                    value: Expression::Cast {
                        target_ty: TypeToken::Primitive(SpanData {
                            span: Span::new_source(source.clone(), 33..37),
                            data: DataType::Char,
                        }),
                        exp: Box::new(Expression::Identifier {
                            ident: Identifier(SpanData {
                                span: Span::new_source(source, 39..44),
                                data: "first".to_string(),
                            }),
                        }),
                    },
                },
            ],
        },
    });

    let result = syntax::parse(tokenized);

    assert_eq!(expected, result);
}
