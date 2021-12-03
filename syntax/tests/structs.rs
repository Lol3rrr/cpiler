use general::{Source, Span, SpanData};
use syntax::{DataType, Identifier, Scope, Statement, StructMembers, TypeToken, AST};

#[test]
fn named_struct_def() {
    let content = "
struct test {
    int first;
    int second;
};
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let mut tokens = tokenizer::tokenize(span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::StructDefinition {
                name: Identifier(SpanData {
                    span: Span::new_source(source.clone(), 8..12),
                    data: "test".to_string(),
                }),
                members: StructMembers {
                    members: vec![
                        (
                            TypeToken::Primitive(SpanData {
                                span: Span::new_source(source.clone(), 19..22),
                                data: DataType::Int,
                            }),
                            Identifier(SpanData {
                                span: Span::new_source(source.clone(), 23..28),
                                data: "first".to_string(),
                            }),
                        ),
                        (
                            TypeToken::Primitive(SpanData {
                                span: Span::new_source(source.clone(), 34..37),
                                data: DataType::Int,
                            }),
                            Identifier(SpanData {
                                span: Span::new_source(source.clone(), 38..44),
                                data: "second".to_string(),
                            }),
                        ),
                    ],
                },
            }],
        },
    });

    let result = syntax::parse(tokens.by_ref());

    assert_eq!(expected, result);
    assert_eq!(None, tokens.next());
}
