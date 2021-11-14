use general::{Span, SpanData};
use syntax::{Expression, Identifier, Scope, Statement, TypeToken, AST};
use tokenizer::DataType;

#[test]
fn simple() {
    let body = "int main() { return 0; }";
    let body_span = Span::from_parts("test", body, 0..body.len());
    let tokenized = tokenizer::tokenize(body_span);

    let expected = AST {
        global_scope: Scope {
            statements: vec![Statement::FunctionDefinition {
                r_type: TypeToken::Primitive(SpanData {
                    span: Span::from_parts("test", "int", 0..3),
                    data: DataType::Int,
                }),
                name: Identifier(SpanData {
                    span: Span::from_parts("test", "main", 4..8),
                    data: "main".to_string(),
                }),
                arguments: vec![],
                body: Scope {
                    statements: vec![Statement::Return(Some(Expression::Literal {
                        content: SpanData {
                            span: Span::from_parts("test", "0", 20..21),
                            data: "0".to_string(),
                        },
                    }))],
                },
            }],
        },
    };

    let result = syntax::parse(tokenized);

    dbg!(&result);

    assert_eq!(expected, result);
}
