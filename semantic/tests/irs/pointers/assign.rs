use general::{arch::Arch, Source, Span};
use ir::{
    BasicBlock, Constant, Expression, FunctionDefinition, Operand, Program, Statement, Type, Value,
    Variable, VariableMetadata,
};

#[test]
fn random_ptr_assign() {
    let content = "
void test() {
    int* x = (int*) 123;
    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_initial = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let x_var = Variable::new("x_17563920617334630623", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::Pointer);
    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::Pointer(Box::new(Type::I32)),
                    base: Operand::Constant(Constant::I64(123)),
                }),
            },
            Statement::SaveVariable { var: x_var },
            Statement::Return(None),
        ],
    );

    func_initial.add_statement(Statement::Jump(func_inner, ir::JumpMetadata::Linear));

    let expected = Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial,
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
fn pointer_to_var() {
    let content = "
void test() {
    int x = 0;
    int* y = &x;
    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_initial = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let x_var = Variable::new("x_973384018644274198", Type::I32);
    let y_var = Variable::new("y_2817108257342626404", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::VarPointer {
            var: Box::new(x_var.name.clone()),
        });

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::SaveVariable { var: x_var.clone() },
            Statement::Assignment {
                target: y_var.clone(),
                value: Value::Expression(Expression::AdressOf {
                    base: Operand::Variable(x_var),
                }),
            },
            Statement::SaveVariable { var: y_var },
            Statement::Return(None),
        ],
    );

    func_initial.add_statement(Statement::Jump(func_inner, ir::JumpMetadata::Linear));

    let expected = Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial,
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
fn pointer_mixed_target() {
    let content = "
void test() {
    int* y = (int*) 0;
    int x = 0;
    y = &x;
    y = (int*) 13;
    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_initial = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let y0_var = Variable::new("y_15711910289717944885", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::Pointer);
    let x_var = Variable::new("x_1617816566900823775", Type::I32);
    let y1_var = y0_var.next_gen().set_meta(VariableMetadata::VarPointer {
        var: Box::new(x_var.name.clone()),
    });
    let y2_var = y1_var.next_gen().set_meta(VariableMetadata::Pointer);

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            Statement::Assignment {
                target: y0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::Pointer(Box::new(Type::I32)),
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::SaveVariable { var: y0_var },
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::SaveVariable { var: x_var.clone() },
            Statement::Assignment {
                target: y1_var.clone(),
                value: Value::Expression(Expression::AdressOf {
                    base: Operand::Variable(x_var),
                }),
            },
            Statement::SaveVariable { var: y1_var },
            Statement::Assignment {
                target: y2_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::Pointer(Box::new(Type::I32)),
                    base: Operand::Constant(Constant::I64(13)),
                }),
            },
            Statement::SaveVariable { var: y2_var },
            Statement::Return(None),
        ],
    );

    func_initial.add_statement(Statement::Jump(func_inner, ir::JumpMetadata::Linear));

    let expected = Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}
