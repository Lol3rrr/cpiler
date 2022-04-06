use general::{Source, Span, SpanData};
use syntax::{Expression, FunctionHead, Identifier, Scope, Statement, TypeToken, AST};
use tokenizer::DataType;

#[test]
fn simple() {
    let body = "int main() { return 0; }";
    let source = Source::new("test", body);
    let body_span: Span = source.clone().into();
    let tokenized = tokenizer::tokenize(body_span);

    let expected = AST {
        global_scope: Scope {
            statements: vec![Statement::FunctionDefinition {
                head: FunctionHead {
                    r_type: TypeToken::Primitive(SpanData {
                        span: Span::new_source(source.clone(), 0..3),
                        data: DataType::Int,
                    }),
                    name: Identifier(SpanData {
                        span: Span::new_source(source.clone(), 4..8),
                        data: "main".to_string(),
                    }),
                    arguments: vec![],
                    var_args: false,
                },
                body: Scope {
                    statements: vec![Statement::Return(Some(Expression::Literal {
                        content: SpanData {
                            span: Span::new_source(source, 20..21),
                            data: "0".to_string(),
                        },
                    }))],
                },
            }],
        },
    };

    let result = syntax::parse(tokenized).unwrap();

    dbg!(&result);

    assert_eq!(expected, result);
}
