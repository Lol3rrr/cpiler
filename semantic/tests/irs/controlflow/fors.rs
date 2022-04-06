use general::{Source, Span};
use ir::{
    BasicBlock, BinaryLogicOp, BinaryOp, Constant, Expression, Operand, PhiEntry, Statement, Type,
    UnaryArithmeticOp, UnaryOp, Value, Variable,
};

#[test]
#[ignore = "At this point i dont really understand what is going on, but it somehow works"]
fn basic_for_loop() {
    let content = "
void test() {
    for (int i = 0; i < 10; i++) {
        int x = i;
    }
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    let var_i0 = Variable::new("i_17473603369839015008", Type::I32);
    let var_i1 = var_i0.next_gen();
    let var_i2 = var_i1.next_gen();
    let var_x = Variable::new("x_5281228482000454764", Type::I32);
    let var_t0 = Variable::tmp(0, Type::I64);
    let var_t1 = Variable::tmp(1, Type::I32);

    let global = BasicBlock::new(vec![], vec![]);

    let func_start = BasicBlock::new(vec![global.weak_ptr()], vec![]);

    let func_second = BasicBlock::new(vec![func_start.weak_ptr()], vec![]);
    func_start.add_statement(Statement::Jump(
        func_second.clone(),
        ir::JumpMetadata::Linear,
    ));

    let setup_block = BasicBlock::new(
        vec![func_second.weak_ptr()],
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
        ],
    );
    func_second.add_statement(Statement::Jump(
        setup_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let loop_inner_block = BasicBlock::new(vec![], vec![]);

    let loop_cond_block = BasicBlock::new(
        vec![setup_block.weak_ptr(), loop_inner_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: var_i1.clone(),
                value: Value::Phi {
                    sources: vec![
                        PhiEntry {
                            var: var_i0,
                            block: setup_block.weak_ptr(),
                        },
                        PhiEntry {
                            var: var_i2.clone(),
                            block: loop_inner_block.weak_ptr(),
                        },
                    ],
                },
            },
            Statement::Assignment {
                target: var_t0.clone(),
                value: Value::Expression(Expression::BinaryOp {
                    op: BinaryOp::Logic(BinaryLogicOp::Less),
                    left: Operand::Variable(var_i1.clone()),
                    right: Operand::Constant(Constant::I64(10)),
                }),
            },
            Statement::JumpTrue(var_t0, loop_inner_block.clone(), ir::JumpMetadata::Linear),
        ],
    );
    setup_block.add_statement(Statement::Jump(
        loop_cond_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    loop_inner_block.set_statements(vec![
        Statement::Assignment {
            target: var_x.clone(),
            value: Value::Variable(var_i1.clone()),
        },
        Statement::SaveVariable { var: var_x },
        Statement::Assignment {
            target: var_t1,
            value: Value::Variable(var_i1.clone()),
        },
        Statement::Assignment {
            target: var_i2.clone(),
            value: Value::Expression(Expression::UnaryOp {
                op: UnaryOp::Arith(UnaryArithmeticOp::Increment),
                base: Operand::Variable(var_i1),
            }),
        },
        Statement::SaveVariable { var: var_i2 },
        Statement::Jump(loop_cond_block.clone(), ir::JumpMetadata::Linear),
    ]);
    loop_inner_block.add_predecessor(loop_cond_block.weak_ptr());

    let loop_end_block = BasicBlock::new(vec![loop_cond_block.weak_ptr()], vec![]);
    loop_cond_block.add_statement(Statement::Jump(
        loop_end_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let func_end_block = BasicBlock::new(vec![loop_end_block.weak_ptr()], vec![]);
    loop_end_block.add_statement(Statement::Jump(func_end_block, ir::JumpMetadata::Linear));

    let expected = ir::Program {
        global,
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

    assert_eq!(expected, result);
}

#[test]
#[ignore = "See test above"]
fn access_after_for_loop() {
    let content = "
void test() {
    int tmp = 0;

    for (int i = 0; i < 10; i++) {
        int x = i;
    }

    int other = tmp;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    todo!()
}
