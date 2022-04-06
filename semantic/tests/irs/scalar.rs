use general::{arch::Arch, Source, Span};
use ir::{
    BasicBlock, BinaryArithmeticOp, BinaryOp, Constant, Expression, FunctionDefinition, Operand,
    Program, Statement, Type, Value, Variable,
};

#[test]
fn simple_scalar() {
    let content = "
void test() {
    long int x = 13;
    long int y = x + 5;
    x = 12;
    y = y * 1;
    return;
}
            ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);
    let global_weak = global_block.weak_ptr();

    let x_var = Variable::new("x_14996960715402433870", Type::I64);
    let x1_var = x_var.next_gen();
    let y_var = Variable::new("y_12304173411602300563", Type::I64);
    let y1_var = y_var.next_gen();

    let initial_block = BasicBlock::new(vec![global_weak], vec![]);
    let initial_weak = initial_block.weak_ptr();
    let inner_block = BasicBlock::new(
        vec![initial_weak],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Constant(Constant::I64(13)),
            },
            Statement::SaveVariable { var: x_var.clone() },
            Statement::Assignment {
                target: y_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(x_var),
                    right: Operand::Constant(Constant::I64(5)),
                }),
            },
            Statement::SaveVariable { var: y_var.clone() },
            Statement::Assignment {
                target: x1_var.clone(),
                value: Value::Constant(Constant::I64(12)),
            },
            Statement::SaveVariable { var: x1_var },
            Statement::Assignment {
                target: y1_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                    left: Operand::Variable(y_var),
                    right: Operand::Constant(Constant::I64(1)),
                }),
            },
            Statement::SaveVariable { var: y1_var },
            Statement::Return(None),
        ],
    );
    initial_block.add_statement(Statement::Jump(inner_block, ir::JumpMetadata::Linear));

    let expected = Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: initial_block,
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
fn scalar_with_args() {
    let input = "
void test(int arg) {
    int x = 13;
    int y = arg + x;
    return;
}
            ";
    let source = Source::new("test", input);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let ast = syntax::parse(tokens).unwrap();
    let aast = semantic::parse(ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let arg_var = Variable::new("arg_7757918524739752622", Type::I32);
    let x_var = Variable::new("x_16654679891977327093", Type::I32);
    let y_var = Variable::new("y_11784424694190178094", Type::I32);

    let func_initial_block = BasicBlock::new(
        vec![global_block.weak_ptr()],
        vec![Statement::Assignment {
            target: arg_var.clone(),
            value: Value::Unknown,
        }],
    );

    let func_body_block = BasicBlock::new(
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
                target: y_var.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                    left: Operand::Variable(arg_var),
                    right: Operand::Variable(x_var),
                }),
            },
            Statement::SaveVariable { var: y_var },
            Statement::Return(None),
        ],
    );
    func_initial_block.add_statement(Statement::Jump(func_body_block, ir::JumpMetadata::Linear));

    let expected = Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![("arg_7757918524739752622".to_string(), Type::I32)],
                return_ty: Type::Void,
                block: func_initial_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = aast.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}
