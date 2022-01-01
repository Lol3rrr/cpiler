use general::{Source, Span};
use ir::{
    BasicBlock, BinaryArithmeticOp, BinaryOp, Constant, Expression, FunctionDefinition, Operand,
    Program, Statement, Type, Value, Variable,
};

#[test]
fn simple_struct_decl() {
    let content = "
struct Test {
    int first;
    char second;
    int third;
};

void test() {
    struct Test x;

    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_initial = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let x_var = Variable::new(
        "x_12621672828407718478",
        ir::Type::Pointer(Box::new(ir::Type::Void)),
    );

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::StackAlloc {
                    size: 12,
                    alignment: 4,
                }),
            },
            Statement::Return(None),
        ],
    );

    func_initial.add_statement(Statement::Jump(func_inner));

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

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn simple_struct_field_assign() {
    let content = "
struct Test {
    int first;
    char second;
    int third;
};

void test() {
    struct Test x;

    x.first = 13;
    x.second = 23;
    x.third = 33;

    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_initial = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let x_var = Variable::new(
        "x_12621672828407718478",
        ir::Type::Pointer(Box::new(ir::Type::Void)),
    );
    let t0_var = Variable::new("__t_0", Type::Pointer(Box::new(Type::Void)));
    let t1_var = Variable::new("__t_1", Type::Pointer(Box::new(Type::Void)));
    let t2_var = Variable::new("__t_2", Type::Pointer(Box::new(Type::Void)));

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::StackAlloc {
                    size: 12,
                    alignment: 4,
                }),
            },
            Statement::Assignment {
                target: t0_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::WriteMemory {
                target: Operand::Variable(t0_var.clone()),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(13)),
                }),
            },
            Statement::Assignment {
                target: t1_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Constant(Constant::I64(4)),
                }),
            },
            Statement::WriteMemory {
                target: Operand::Variable(t1_var.clone()),
                value: Value::Expression(Expression::Cast {
                    target: Type::I8,
                    base: Operand::Constant(Constant::I64(23)),
                }),
            },
            Statement::Assignment {
                target: t2_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Constant(Constant::I64(8)),
                }),
            },
            Statement::WriteMemory {
                target: Operand::Variable(t2_var.clone()),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(33)),
                }),
            },
            Statement::Return(None),
        ],
    );

    func_initial.add_statement(Statement::Jump(func_inner));

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

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}
#[test]
fn struct_field_read_assign() {
    let content = "
struct Test {
    int first;
    char second;
    int third;
};

void test() {
    struct Test x;

    x.first += 13;

    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_initial = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let x_var = Variable::new(
        "x_12621672828407718478",
        Type::Pointer(Box::new(Type::Void)),
    );
    let t0_var = Variable::new("__t_0", Type::Pointer(Box::new(Type::Void)));
    let t1_var = Variable::new("__t_1", Type::Pointer(Box::new(Type::Void)));
    let t2_var = Variable::new("__t_2", Type::I32);
    let t3_var = Variable::new("__t_3", Type::I64);
    let t4_var = Variable::new("__t_4", Type::I64);

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            // This is the Address of the Struct on the Stack
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::StackAlloc {
                    size: 12,
                    alignment: 4,
                }),
            },
            // This is the Target address of the Field we want to assign a new Value to
            Statement::Assignment {
                target: t0_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Constant(Constant::I64(0)),
                }),
            },
            // This is the Address of the Field we want to read
            Statement::Assignment {
                target: t1_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Constant(Constant::I64(0)),
                }),
            },
            // This reads the Value from the Field
            Statement::Assignment {
                target: t2_var.clone(),
                value: Value::Expression(Expression::ReadMemory {
                    address: Operand::Variable(t1_var.clone()),
                    read_ty: Type::I32,
                }),
            },
            // Cast the Read Value to an I64
            Statement::Assignment {
                target: t3_var.clone(),
                value: Value::Expression(Expression::Cast {
                    base: Operand::Variable(t2_var.clone()),
                    target: Type::I64,
                }),
            },
            // Add the 13 to the Value
            Statement::Assignment {
                target: t4_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(t3_var.clone()),
                    right: Operand::Constant(Constant::I64(13)),
                }),
            },
            // Store the new Value back into the target Field
            Statement::WriteMemory {
                target: Operand::Variable(t0_var.clone()),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Variable(t4_var.clone()),
                }),
            },
            Statement::Return(None),
        ],
    );

    func_initial.add_statement(Statement::Jump(func_inner));

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

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn array_of_struct_access() {
    let content = "
struct Test {
    int first;
};

void test() {
    struct Test x[5];

    x[0].first = 13;

    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_initial = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let x_var = Variable::new(
        "x_10573110594158173564",
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Void)))),
    );
    let t0_var = Variable::new("__t_0", Type::I64);
    let t1_var = Variable::new(
        "__t_1",
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Void)))),
    );
    let t2_var = Variable::new(
        "__t_2",
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Void)))),
    );

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::StackAlloc {
                    size: 20,
                    alignment: 4,
                }),
            },
            Statement::Assignment {
                target: t0_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Constant(Constant::I64(0)),
                    right: Operand::Constant(Constant::I64(4)),
                }),
            },
            Statement::Assignment {
                target: t1_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Variable(t0_var.clone()),
                }),
            },
            Statement::Assignment {
                target: t2_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(t1_var.clone()),
                    right: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::WriteMemory {
                target: Operand::Variable(t2_var.clone()),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(13)),
                }),
            },
            Statement::Return(None),
        ],
    );

    func_initial.add_statement(Statement::Jump(func_inner));

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

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn array_of_struct_read() {
    let content = "
struct Test {
    int first;
};

void test() {
    struct Test x[5];

    int tmp = x[0].first;

    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_initial = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let x_var = Variable::new(
        "x_10573110594158173564",
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Void)))),
    );
    let t0_var = Variable::new("__t_0", Type::I64);
    let t1_var = Variable::new(
        "__t_1",
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Void)))),
    );
    let t2_var = Variable::new(
        "__t_2",
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Void)))),
    );
    let tmp_var = Variable::new("tmp_14764528945755187508", Type::I32);

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::StackAlloc {
                    size: 20,
                    alignment: 4,
                }),
            },
            Statement::Assignment {
                target: t0_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Constant(Constant::I64(0)),
                    right: Operand::Constant(Constant::I64(4)),
                }),
            },
            Statement::Assignment {
                target: t1_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Variable(t0_var.clone()),
                }),
            },
            Statement::Assignment {
                target: t2_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(t1_var.clone()),
                    right: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::Assignment {
                target: tmp_var.clone(),
                value: Value::Expression(Expression::ReadMemory {
                    address: Operand::Variable(t2_var.clone()),
                    read_ty: Type::I32,
                }),
            },
            Statement::Return(None),
        ],
    );

    func_initial.add_statement(Statement::Jump(func_inner));

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

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
#[ignore = "Does not support nested Arrays yet"]
fn nested_array_of_struct_access() {
    let content = "
struct Test {
    int first;
};

void test() {
    struct Test x[5][5];

    x[0][0].first = 13;

    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_initial = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let x_var = Variable::new(
        "x",
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Void)))),
    );
    let t0_var = Variable::new("__t_0", Type::I64);
    let t1_var = Variable::new(
        "__t_1",
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Void)))),
    );
    let t2_var = Variable::new(
        "__t_2",
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Void)))),
    );

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::StackAlloc {
                    size: 20,
                    alignment: 4,
                }),
            },
            Statement::Assignment {
                target: t0_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Constant(Constant::I64(0)),
                    right: Operand::Constant(Constant::I64(4)),
                }),
            },
            Statement::Assignment {
                target: t1_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Variable(t0_var.clone()),
                }),
            },
            Statement::Assignment {
                target: t2_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(t1_var.clone()),
                    right: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::WriteMemory {
                target: Operand::Variable(t2_var.clone()),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(13)),
                }),
            },
            Statement::Return(None),
        ],
    );

    func_initial.add_statement(Statement::Jump(func_inner));

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

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}
