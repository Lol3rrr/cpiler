use general::{arch::Arch, Source, Span};
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
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let x_var = Variable::new("x_17227083962473714508", Type::I32);
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
                target: x_var.clone(),
                value: Value::Variable(t0_var),
            },
            Statement::SaveVariable { var: x_var },
            Statement::Return(None),
        ],
    );
    func_block.add_statement(Statement::Jump(func_inner_block, ir::JumpMetadata::Linear));

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

    let result = input.convert_to_ir(Arch::X86_64);
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
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let x_var = Variable::new("x_3421002591759186305", Type::I32);
    let y_var = Variable::new("y_6298470074920897498", Type::I32);
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
            Statement::SaveVariable { var: x_var.clone() },
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
                    arguments: vec![Operand::Variable(x_var), Operand::Variable(t0_var)],
                }),
            },
            Statement::Assignment {
                target: y_var.clone(),
                value: Value::Variable(t1_var),
            },
            Statement::SaveVariable { var: y_var },
            Statement::Return(None),
        ],
    );
    func_initial_block.add_statement(Statement::Jump(func_inner_block, ir::JumpMetadata::Linear));

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

    let result = input.convert_to_ir(Arch::X86_64);
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
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let x_var = Variable::new("x_9131165162366007046", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::Pointer);
    let x_2_var = x_var.next_gen();
    let y_var = Variable::new("y_516652289463271765", Type::I32);
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
            Statement::SaveVariable { var: x_var.clone() },
            Statement::Assignment {
                target: t0_var.clone(),
                value: Value::Expression(Expression::FunctionCall {
                    name: "other".to_string(),
                    return_ty: Type::I32,
                    arguments: vec![Operand::Variable(x_var)],
                }),
            },
            Statement::Assignment {
                target: x_2_var,
                value: Value::Unknown,
            },
            Statement::Assignment {
                target: y_var.clone(),
                value: Value::Variable(t0_var),
            },
            Statement::SaveVariable { var: y_var },
            Statement::Return(None),
        ],
    );
    func_initial_block.add_statement(Statement::Jump(func_inner_block, ir::JumpMetadata::Linear));

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

    let result = input.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn raw_function_call_no_args() {
    let content = "
int other();

void test() {
    other();
    return;
}
            ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_block = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let func_inner_block = BasicBlock::new(
        vec![func_block.weak_ptr()],
        vec![
            Statement::Call {
                name: "other".to_string(),
                arguments: vec![],
            },
            Statement::Return(None),
        ],
    );
    func_block.add_statement(Statement::Jump(func_inner_block, ir::JumpMetadata::Linear));

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

    let result = input.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}
