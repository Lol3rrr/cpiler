use general::{arch::Arch, Source, Span};
use ir::{
    BasicBlock, Constant, Expression, FunctionDefinition, Operand, PhiEntry, Program, Statement,
    Type, Value, Variable,
};

#[test]
#[ignore = "Figure out a better way to verify semantics"]
fn single_if() {
    let content = "
void test() {
    int x = 0;
    if (2) {
        x = 2;
    }
    
    int y = x;

    return;
}
            ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let x_var = Variable::new("x_973384018644274198", Type::I32);
    let t0_var = Variable::new("__t_0", Type::I64);
    let x1_var = x_var.next_gen();
    let x2_var = x_var.next_gen();
    let y_var = Variable::new("y_13143486146492289088", Type::I32);

    let function_arg_block = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let function_first_block = BasicBlock::new(
        vec![function_arg_block.weak_ptr()],
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
                target: t0_var.clone(),
                value: Value::Constant(Constant::I64(2)),
            },
        ],
    );
    function_arg_block.add_statement(Statement::Jump(
        function_first_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let inner_if_block = BasicBlock::new(
        vec![function_first_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x1_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(2)),
                }),
            },
            Statement::SaveVariable {
                var: x1_var.clone(),
            },
        ],
    );
    function_first_block.add_statement(Statement::JumpTrue(
        t0_var,
        inner_if_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let end_block = BasicBlock::new(
        vec![inner_if_block.weak_ptr(), function_first_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x2_var.clone(),
                value: Value::Phi {
                    sources: vec![
                        PhiEntry {
                            block: function_first_block.weak_ptr(),
                            var: x1_var,
                        },
                        PhiEntry {
                            block: inner_if_block.weak_ptr(),
                            var: x_var,
                        },
                    ],
                },
            },
            Statement::Assignment {
                target: y_var.clone(),
                value: Value::Variable(x2_var),
            },
            Statement::SaveVariable { var: y_var },
            Statement::Return(None),
        ],
    );
    function_first_block
        .add_statement(Statement::Jump(end_block.clone(), ir::JumpMetadata::Linear));
    inner_if_block.add_statement(Statement::Jump(end_block, ir::JumpMetadata::Linear));

    let expected = Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: function_arg_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    std::fs::write("./result.dot", result.to_dot()).unwrap();
    std::fs::write("./expected.dot", expected.to_dot()).unwrap();

    assert_eq!(expected, result);
}

#[test]
#[ignore = "Figure out a better way to verify semantics"]
fn if_else_blocks() {
    let content = "
void test() {
    int x = 0;
    if (2) {
        x = 2;
    } else {
        x = 3;
    }
    
    int y = x;

    return;
}
            ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let function_start_block = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let x0_var = Variable::new("x_973384018644274198", Type::I32);
    let x1_var = x0_var.next_gen();
    let x2_var = x1_var.next_gen();
    let x3_var = x2_var.next_gen();
    let t0_var = Variable::new("__t_0", Type::I64);
    let y_var = Variable::new("y_9205855845031901256", Type::I32);

    let function_first = BasicBlock::new(
        vec![function_start_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::SaveVariable { var: x0_var },
            Statement::Assignment {
                target: t0_var.clone(),
                value: Value::Constant(Constant::I64(2)),
            },
        ],
    );
    function_start_block.add_statement(Statement::Jump(
        function_first.clone(),
        ir::JumpMetadata::Linear,
    ));

    let true_block = BasicBlock::new(
        vec![function_first.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x1_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(2)),
                }),
            },
            Statement::SaveVariable {
                var: x1_var.clone(),
            },
        ],
    );
    function_first.add_statement(Statement::JumpTrue(
        t0_var,
        true_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let false_block = BasicBlock::new(
        vec![function_first.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x2_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(3)),
                }),
            },
            Statement::SaveVariable {
                var: x2_var.clone(),
            },
        ],
    );
    function_first.add_statement(Statement::Jump(
        false_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let function_second = BasicBlock::new(
        vec![true_block.weak_ptr(), false_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x3_var.clone(),
                value: Value::Phi {
                    sources: vec![
                        PhiEntry {
                            block: true_block.weak_ptr(),
                            var: x1_var,
                        },
                        PhiEntry {
                            block: false_block.weak_ptr(),
                            var: x2_var,
                        },
                    ],
                },
            },
            Statement::Assignment {
                target: y_var.clone(),
                value: Value::Variable(x3_var),
            },
            Statement::SaveVariable { var: y_var },
            Statement::Return(None),
        ],
    );

    true_block.add_statement(Statement::Jump(
        function_second.clone(),
        ir::JumpMetadata::Linear,
    ));
    false_block.add_statement(Statement::Jump(function_second, ir::JumpMetadata::Linear));

    let expected = Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: function_start_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    std::fs::write("./result.dot", result.to_dot()).unwrap();
    std::fs::write("./expected.dot", expected.to_dot()).unwrap();

    assert_eq!(expected, result);
}
