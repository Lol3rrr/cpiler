use general::{Source, Span};
use ir::{
    BasicBlock, BinaryArithmeticOp, BinaryOp, Constant, Expression, FunctionDefinition, Operand,
    Program, Statement, Type, UnaryArithmeticOp, UnaryOp, Value, Variable, VariableMetadata,
};

#[test]
fn simple_array() {
    let content = "
void test() {
    int x[10];

    x[0] = 13;
    x[1] = 23;

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

    let x_var = Variable::new("x_973384018644274198", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::Pointer);
    let t0_var = Variable::new("__t_0", Type::I64);
    let t1_var = Variable::new("__t_1", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::Pointer);
    let t2_var = Variable::new("__t_2", Type::I64);
    let t3_var = Variable::new("__t_3", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::Pointer);

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::StackAlloc {
                    size: 10 * 4,
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
            Statement::WriteMemory {
                target: Operand::Variable(t1_var.clone()),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(13)),
                }),
            },
            Statement::Assignment {
                target: t2_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Constant(Constant::I64(1)),
                    right: Operand::Constant(Constant::I64(4)),
                }),
            },
            Statement::Assignment {
                target: t3_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Variable(t2_var.clone()),
                }),
            },
            Statement::WriteMemory {
                target: Operand::Variable(t3_var.clone()),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(23)),
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
fn array_suffix_increment_decrement() {
    let content = "
void test() {
    int x[10];

    x[0] = 13;
    int tmp1 = x[0]++;
    int tmp2 = x[0]--;

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

    // The start of the Array
    let x_var = Variable::new("x_973384018644274198", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::Pointer);

    // The Variables needed for the first assignment
    let t0_var = Variable::new("__t_0", Type::I64);
    let t1_var = Variable::new("__t_1", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::Pointer);

    // The Variables needed for the second Assignment + Update
    let tmp1_var = Variable::new("tmp1_9656661383669412478", Type::I32);
    let t2_var = Variable::new("__t_2", Type::I64);
    let t3_var = Variable::new("__t_3", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::Pointer);
    let t4_var = Variable::tmp(4, Type::I32);
    let t5_var = Variable::tmp(5, Type::I64);
    let t6_var =
        Variable::tmp(6, Type::Pointer(Box::new(Type::I32))).set_meta(VariableMetadata::Pointer);
    let t7_var = Variable::tmp(7, Type::I32);
    let t8_var = Variable::tmp(8, Type::I64);
    let t9_var =
        Variable::tmp(9, Type::Pointer(Box::new(Type::I32))).set_meta(VariableMetadata::Pointer);

    // The Variables needed for the third Assignment + Update
    let tmp2_var = Variable::new("tmp2_2108483379725238258", Type::I32);
    let t10_var = Variable::tmp(10, Type::I64);
    let t11_var =
        Variable::tmp(11, Type::Pointer(Box::new(Type::I32))).set_meta(VariableMetadata::Pointer);
    let t12_var = Variable::tmp(12, Type::I32);
    let t13_var = Variable::tmp(13, Type::I64);
    let t14_var =
        Variable::tmp(14, Type::Pointer(Box::new(Type::I32))).set_meta(VariableMetadata::Pointer);
    let t15_var = Variable::tmp(15, Type::I32);
    let t16_var = Variable::tmp(16, Type::I64);
    let t17_var =
        Variable::tmp(17, Type::Pointer(Box::new(Type::I32))).set_meta(VariableMetadata::Pointer);

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            // This is the first Assignment "x[0] = 13"
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::StackAlloc {
                    size: 10 * 4,
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
            Statement::WriteMemory {
                target: Operand::Variable(t1_var.clone()),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(13)),
                }),
            },
            // This is related to the first Assign + Update
            Statement::Assignment {
                target: t2_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Constant(Constant::I64(0)),
                    right: Operand::Constant(Constant::I64(4)),
                }),
            },
            Statement::Assignment {
                target: t3_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Variable(t2_var.clone()),
                }),
            },
            Statement::Assignment {
                target: t4_var.clone(),
                value: Value::Expression(Expression::ReadMemory {
                    address: Operand::Variable(t3_var.clone()),
                    read_ty: Type::I32,
                }),
            },
            Statement::Assignment {
                target: t5_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Constant(Constant::I64(0)),
                    right: Operand::Constant(Constant::I64(4)),
                }),
            },
            Statement::Assignment {
                target: t6_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Variable(t5_var.clone()),
                }),
            },
            Statement::Assignment {
                target: t7_var.clone(),
                value: Value::Expression(Expression::ReadMemory {
                    address: Operand::Variable(t6_var.clone()),
                    read_ty: Type::I32,
                }),
            },
            Statement::Assignment {
                target: t8_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Constant(Constant::I64(0)),
                    right: Operand::Constant(Constant::I64(4)),
                }),
            },
            Statement::Assignment {
                target: t9_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Variable(t8_var.clone()),
                }),
            },
            Statement::WriteMemory {
                target: Operand::Variable(t9_var.clone()),
                value: Value::Expression(Expression::UnaryOp {
                    op: UnaryOp::Arith(UnaryArithmeticOp::Increment),
                    base: Operand::Variable(t7_var.clone()),
                }),
            },
            Statement::Assignment {
                target: tmp1_var.clone(),
                value: Value::Variable(t4_var.clone()),
            },
            Statement::SaveVariable {
                var: tmp1_var.clone(),
            },
            // The second Update + Assignment
            Statement::Assignment {
                target: t10_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Constant(Constant::I64(0)),
                    right: Operand::Constant(Constant::I64(4)),
                }),
            },
            Statement::Assignment {
                target: t11_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Variable(t10_var.clone()),
                }),
            },
            Statement::Assignment {
                target: t12_var.clone(),
                value: Value::Expression(Expression::ReadMemory {
                    address: Operand::Variable(t11_var.clone()),
                    read_ty: Type::I32,
                }),
            },
            Statement::Assignment {
                target: t13_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Constant(Constant::I64(0)),
                    right: Operand::Constant(Constant::I64(4)),
                }),
            },
            Statement::Assignment {
                target: t14_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Variable(t13_var.clone()),
                }),
            },
            Statement::Assignment {
                target: t15_var.clone(),
                value: Value::Expression(Expression::ReadMemory {
                    address: Operand::Variable(t14_var.clone()),
                    read_ty: Type::I32,
                }),
            },
            Statement::Assignment {
                target: t16_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Constant(Constant::I64(0)),
                    right: Operand::Constant(Constant::I64(4)),
                }),
            },
            Statement::Assignment {
                target: t17_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var.clone()),
                    right: Operand::Variable(t16_var.clone()),
                }),
            },
            Statement::WriteMemory {
                target: Operand::Variable(t17_var.clone()),
                value: Value::Expression(Expression::UnaryOp {
                    op: UnaryOp::Arith(UnaryArithmeticOp::Decrement),
                    base: Operand::Variable(t15_var.clone()),
                }),
            },
            Statement::Assignment {
                target: tmp2_var.clone(),
                value: Value::Variable(t12_var.clone()),
            },
            Statement::SaveVariable {
                var: tmp2_var.clone(),
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
