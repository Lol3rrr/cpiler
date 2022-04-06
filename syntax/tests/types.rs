use general::{Source, Span, SpanData};
use syntax::{DataType, Identifier, Modifier, Scope, Statement, TypeToken, AST};

#[test]
fn long_long_int() {
    let input = "
long long int x;
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let mut tokens = tokenizer::tokenize(span);

    let expected = Ok(AST {
        global_scope: Scope {
            statements: vec![Statement::VariableDeclaration {
                ty: TypeToken::Composition {
                    modifier: SpanData {
                        span: Span::new_source(source.clone(), 1..5),
                        data: Modifier::Long,
                    },
                    base: Box::new(TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 6..10),
                        data: DataType::Long,
                    })),
                },
                name: Identifier(SpanData {
                    span: Span::new_source(source, 15..16),
                    data: "x".to_string(),
                }),
            }],
        },
    });

    let result = syntax::parse(tokens.by_ref());

    assert_eq!(expected, result);
    assert_eq!(None, tokens.next());
}
