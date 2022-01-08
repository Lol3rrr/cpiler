use general::{Source, Span};
use ir::{
    BasicBlock, Constant, Expression, Operand, Statement, Type, UnaryArithmeticOp, UnaryOp, Value,
    Variable,
};

#[test]
fn standalone_suffix_increment() {
    let content = "
void test() {
    int i = 0;
    i++;
}
            ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let var_i0 = Variable::new("i_13491578160084837238", Type::I32);
    let var_i1 = var_i0.next_gen();
    let var_t0 = Variable::tmp(0, Type::I32);

    let global_block = BasicBlock::new(vec![], vec![]);

    let func_start = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let func_content = BasicBlock::new(
        vec![func_start.weak_ptr()],
        vec![
            Statement::Assignment {
                target: var_i0.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::SaveVariable {
                var: var_i0.clone(),
            },
            Statement::Assignment {
                target: var_t0.clone(),
                value: Value::Variable(var_i0.clone()),
            },
            Statement::Assignment {
                target: var_i1.clone(),
                value: Value::Expression(Expression::UnaryOp {
                    op: UnaryOp::Arith(UnaryArithmeticOp::Increment),
                    base: Operand::Variable(var_i0.clone()),
                }),
            },
            Statement::SaveVariable {
                var: var_i1.clone(),
            },
        ],
    );
    func_start.add_statement(Statement::Jump(func_content));

    let expected = ir::Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            ir::FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_start,
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
fn assign_suffix_increment() {
    let content = "
void test() {
    int i = 0;
    int x = i++;
}
            ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let var_i0 = Variable::new("i_13491578160084837238", Type::I32);
    let var_i1 = var_i0.next_gen();
    let var_t0 = Variable::tmp(0, Type::I32);
    let var_x = Variable::new("x_742235982404886134", Type::I32);

    let global_block = BasicBlock::new(vec![], vec![]);

    let func_start = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let func_content = BasicBlock::new(
        vec![func_start.weak_ptr()],
        vec![
            Statement::Assignment {
                target: var_i0.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::SaveVariable {
                var: var_i0.clone(),
            },
            Statement::Assignment {
                target: var_t0.clone(),
                value: Value::Variable(var_i0.clone()),
            },
            Statement::Assignment {
                target: var_i1.clone(),
                value: Value::Expression(Expression::UnaryOp {
                    op: UnaryOp::Arith(UnaryArithmeticOp::Increment),
                    base: Operand::Variable(var_i0.clone()),
                }),
            },
            Statement::SaveVariable {
                var: var_i1.clone(),
            },
            Statement::Assignment {
                target: var_x.clone(),
                value: Value::Variable(var_t0.clone()),
            },
            Statement::SaveVariable { var: var_x.clone() },
        ],
    );
    func_start.add_statement(Statement::Jump(func_content));

    let expected = ir::Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            ir::FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_start,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}
