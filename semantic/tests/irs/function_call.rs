use general::{Source, Span};
use ir::{
    BasicBlock, Constant, Expression, FunctionDefinition, Operand, Statement, Type, Value,
    Variable, VariableMetadata,
};

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

#[test]
fn call_with_scalar_args() {
    let content = "
int other(int first, int second);

void test() {
    int x = 13;

    int y = other(x, 23);
    
    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let x_var = Variable::new("x", Type::I32);
    let y_var = Variable::new("y", Type::I32);
    let t0_var = Variable::tmp(0, Type::I32);
    let t1_var = Variable::tmp(1, Type::I32);

    let global = BasicBlock::initial(vec![]);

    let func_initial_block = BasicBlock::new(vec![global.weak_ptr()], vec![]);

    let func_inner_block = BasicBlock::new(
        vec![func_initial_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(13)),
                }),
            },
            Statement::Assignment {
                target: t0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(23)),
                }),
            },
            Statement::Assignment {
                target: t1_var.clone(),
                value: Value::Expression(Expression::FunctionCall {
                    name: "other".to_string(),
                    return_ty: Type::I32,
                    arguments: vec![
                        Operand::Variable(x_var.clone()),
                        Operand::Variable(t0_var.clone()),
                    ],
                }),
            },
            Statement::Assignment {
                target: y_var.clone(),
                value: Value::Variable(t1_var.clone()),
            },
            Statement::Return(None),
        ],
    );
    func_initial_block.add_statement(Statement::Jump(func_inner_block));

    let expected = ir::Program {
        global,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: ir::Type::Void,
                block: func_initial_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir();
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn call_with_ptr_arg() {
    let content = "
int other(int* first);

void test() {
    int* x = (int*) 13;

    int y = other(x);
    
    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let mut x_var = Variable::new("x", Type::Pointer(Box::new(Type::I32)));
    x_var.meta = VariableMetadata::Pointer;
    let x_2_var = x_var.next_gen();
    let y_var = Variable::new("y", Type::I32);
    let t0_var = Variable::tmp(0, Type::I32);

    let global = BasicBlock::initial(vec![]);

    let func_initial_block = BasicBlock::new(vec![global.weak_ptr()], vec![]);

    let func_inner_block = BasicBlock::new(
        vec![func_initial_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::Pointer(Box::new(Type::I32)),
                    base: Operand::Constant(Constant::I64(13)),
                }),
            },
            Statement::Assignment {
                target: t0_var.clone(),
                value: Value::Expression(Expression::FunctionCall {
                    name: "other".to_string(),
                    return_ty: Type::I32,
                    arguments: vec![Operand::Variable(x_var.clone())],
                }),
            },
            Statement::Assignment {
                target: x_2_var.clone(),
                value: Value::Unknown,
            },
            Statement::Assignment {
                target: y_var.clone(),
                value: Value::Variable(t0_var.clone()),
            },
            Statement::Return(None),
        ],
    );
    func_initial_block.add_statement(Statement::Jump(func_inner_block));

    let expected = ir::Program {
        global,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: ir::Type::Void,
                block: func_initial_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir();
    dbg!(&result);

    assert_eq!(expected, result);
}
