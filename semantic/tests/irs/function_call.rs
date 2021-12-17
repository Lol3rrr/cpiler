use general::{Source, Span};
use ir::{BasicBlock, Expression, FunctionDefinition, Statement, Type, Value, Variable};

#[test]
fn function_call_no_args() {
    let content = "
int other();

void test() {
    int x = other();
    return;
}
            ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let x_var = Variable::new("x", Type::I32);
    let t0_var = Variable::tmp(0, Type::I32);

    let func_block = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let func_inner_block = BasicBlock::new(
        vec![func_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: t0_var.clone(),
                value: Value::Expression(Expression::FunctionCall {
                    name: "other".to_string(),
                    arguments: vec![],
                    return_ty: Type::I32,
                }),
            },
            Statement::Assignment {
                target: x_var,
                value: Value::Variable(t0_var),
            },
            Statement::Return(None),
        ],
    );
    func_block.add_statement(Statement::Jump(func_inner_block));

    let expected = ir::Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir();
    dbg!(&result);

    assert_eq!(expected, result);
}
