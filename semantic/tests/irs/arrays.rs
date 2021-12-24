use general::{Source, Span};
use ir::{
    BasicBlock, BinaryArithmeticOp, BinaryOp, Constant, Expression, FunctionDefinition, Operand,
    Program, Statement, Type, Value, Variable, VariableMetadata,
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

    let x_var =
        Variable::new("x", Type::Pointer(Box::new(Type::I32))).set_meta(VariableMetadata::Pointer);
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
