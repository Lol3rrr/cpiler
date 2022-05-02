use std::path::{Path, PathBuf};

use general::{arch::Arch, Source, Span};
use ir::{
    BasicBlock, BinaryArithmeticOp, BinaryOp, Constant, Expression, FunctionDefinition, Operand,
    PhiEntry, Program, Statement, Type, UnaryArithmeticOp, UnaryOp, Value, Variable,
};

#[test]
#[ignore = "Figure out a better way to verify semantics"]
fn simple_while_loop() {
    let input = "
void test() {
    while (2) {
        int x = 0;
    }

    return;
}
        ";
    let source = Source::new("test", input);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global = BasicBlock::initial(vec![]);

    let t0_var = Variable::new("__t_0", Type::I64);
    let x0_var = Variable::new("x_13661097461077092700", Type::I32);

    let func_initial_block = BasicBlock::new(vec![global.weak_ptr()], vec![]);

    let func_first_block = BasicBlock::new(vec![func_initial_block.weak_ptr()], vec![]);
    func_initial_block.add_statement(Statement::Jump(
        func_first_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let loop_cond_block = BasicBlock::new(
        vec![func_first_block.weak_ptr()],
        vec![Statement::Assignment {
            target: t0_var.clone(),
            value: Value::Constant(Constant::I64(2)),
        }],
    );
    func_first_block.add_statement(Statement::Jump(
        loop_cond_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let loop_inner_block = BasicBlock::new(
        vec![loop_cond_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::SaveVariable { var: x0_var },
            Statement::Jump(loop_cond_block.clone(), ir::JumpMetadata::Linear),
        ],
    );
    loop_cond_block.add_statement(Statement::JumpTrue(
        t0_var,
        loop_inner_block.clone(),
        ir::JumpMetadata::Linear,
    ));
    loop_cond_block.add_predecessor(loop_inner_block.weak_ptr());

    let func_end_block = BasicBlock::new(
        vec![loop_cond_block.weak_ptr()],
        vec![Statement::Return(None)],
    );
    loop_cond_block.add_statement(Statement::Jump(func_end_block, ir::JumpMetadata::Linear));

    let expected = Program {
        global,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    let test_file_path = {
        let raw = Path::new(file!());
        let mut result = PathBuf::new();
        for part in raw.components().skip(1) {
            result.push(part);
        }

        result
    };
    let test_dir_path = test_file_path.parent().unwrap();

    let result_path = test_dir_path.join("simple-result.dot");
    std::fs::write(result_path, result.to_dot()).unwrap();

    assert_eq!(expected, result);
}

#[test]
#[ignore = "To bored and confused"]
fn while_loop_modifying_in_cond_inner() {
    let input = "
void test() {
    int x = 0;
    while (x--) {
        x = x - 1;
    }

    return;
}
        ";
    let source = Source::new("test", input);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global = BasicBlock::initial(vec![]);

    let t0_var = Variable::new("__t_0", Type::I32);
    let x0_var = Variable::new("x_973384018644274198", Type::I32);
    let x1_var = x0_var.next_gen();
    let x2_var = x1_var.next_gen();
    let x3_var = x2_var.next_gen();
    let t1_var = Variable::tmp(1, Type::I64);
    let t2_var = Variable::tmp(2, Type::I64);
    let t3_var = Variable::tmp(3, Type::I64);

    let func_initial_block = BasicBlock::new(vec![global.weak_ptr()], vec![]);

    let func_first_block = BasicBlock::new(
        vec![func_initial_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::SaveVariable {
                var: x0_var.clone(),
            },
        ],
    );
    func_initial_block.add_statement(Statement::Jump(
        func_first_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let loop_inner_block = BasicBlock::new(vec![], vec![]);
    let loop_cond_block = BasicBlock::new(
        vec![func_first_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x1_var.clone(),
                value: Value::Phi {
                    sources: vec![
                        PhiEntry {
                            var: x0_var,
                            block: func_first_block.weak_ptr(),
                        },
                        PhiEntry {
                            var: x3_var.clone(),
                            block: loop_inner_block.weak_ptr(),
                        },
                    ],
                },
            },
            Statement::Assignment {
                target: t0_var.clone(),
                value: Value::Variable(x1_var.clone()),
            },
            Statement::Assignment {
                target: x2_var.clone(),
                value: Value::Expression(Expression::UnaryOp {
                    op: UnaryOp::Arith(UnaryArithmeticOp::Decrement),
                    base: Operand::Variable(x1_var),
                }),
            },
            Statement::SaveVariable {
                var: x2_var.clone(),
            },
            Statement::Assignment {
                target: t1_var.clone(),
                value: Value::Variable(t0_var),
            },
            Statement::JumpTrue(t1_var, loop_inner_block.clone(), ir::JumpMetadata::Linear),
        ],
    );
    func_first_block.add_statement(Statement::Jump(
        loop_cond_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    loop_inner_block.add_predecessor(loop_inner_block.weak_ptr());
    loop_inner_block.set_statements(vec![
        Statement::Assignment {
            target: t2_var.clone(),
            value: Value::Expression(Expression::Cast {
                target: Type::I64,
                base: Operand::Variable(x2_var),
            }),
        },
        Statement::Assignment {
            target: t3_var.clone(),
            value: Value::Expression(Expression::BinaryOp {
                op: BinaryOp::Arith(BinaryArithmeticOp::Sub),
                left: Operand::Variable(t2_var),
                right: Operand::Constant(Constant::I64(1)),
            }),
        },
        Statement::Assignment {
            target: x3_var.clone(),
            value: Value::Expression(Expression::Cast {
                target: Type::I32,
                base: Operand::Variable(t3_var),
            }),
        },
        Statement::SaveVariable { var: x3_var },
        Statement::Jump(loop_cond_block.clone(), ir::JumpMetadata::Linear),
    ]);
    loop_cond_block.add_predecessor(loop_inner_block.weak_ptr());

    let func_end_block = BasicBlock::new(
        vec![loop_cond_block.weak_ptr()],
        vec![Statement::Return(None)],
    );
    loop_cond_block.add_statement(Statement::Jump(func_end_block, ir::JumpMetadata::Linear));

    let expected = Program {
        global,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    let test_file_path = {
        let raw = Path::new(file!());
        let mut result = PathBuf::new();
        for part in raw.components().skip(1) {
            result.push(part);
        }

        result
    };
    let test_dir_path = test_file_path.parent().unwrap();

    let result_path = test_dir_path.join("modifying-result.dot");
    std::fs::write(result_path, result.to_dot()).unwrap();

    assert_eq!(expected, result);
}

#[test]
#[ignore = "Figure out a better way to verify semantics"]
fn while_loop_with_break() {
    let input = "
void test() {
    while (2) {
        int x = 0;
        break;
    }

    return;
}
        ";
    let source = Source::new("test", input);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global = BasicBlock::initial(vec![]);

    let t0_var = Variable::new("__t_0", Type::I64);
    let x0_var = Variable::new("x_13661097461077092700", Type::I32);

    let func_initial_block = BasicBlock::new(vec![global.weak_ptr()], vec![]);

    let func_first_block = BasicBlock::new(vec![func_initial_block.weak_ptr()], vec![]);
    func_initial_block.add_statement(Statement::Jump(
        func_first_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let loop_cond_block = BasicBlock::new(
        vec![func_first_block.weak_ptr()],
        vec![Statement::Assignment {
            target: t0_var.clone(),
            value: Value::Constant(Constant::I64(2)),
        }],
    );
    func_first_block.add_statement(Statement::Jump(
        loop_cond_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let func_end_block = BasicBlock::new(vec![], vec![Statement::Return(None)]);

    let loop_inner_block = BasicBlock::new(
        vec![loop_cond_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::SaveVariable { var: x0_var },
            Statement::Jump(func_end_block.clone(), ir::JumpMetadata::Linear),
            Statement::Jump(loop_cond_block.clone(), ir::JumpMetadata::Linear),
        ],
    );
    loop_cond_block.add_statement(Statement::JumpTrue(
        t0_var,
        loop_inner_block.clone(),
        ir::JumpMetadata::Linear,
    ));
    loop_cond_block.add_predecessor(loop_inner_block.weak_ptr());
    loop_cond_block.add_statement(Statement::Jump(
        func_end_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    func_end_block.add_predecessor(loop_cond_block.weak_ptr());
    func_end_block.add_predecessor(loop_inner_block.weak_ptr());

    let expected = Program {
        global,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    dbg!(&expected);

    let test_file_path = {
        let raw = Path::new(file!());
        let mut result = PathBuf::new();
        for part in raw.components().skip(1) {
            result.push(part);
        }

        result
    };
    let test_dir_path = test_file_path.parent().unwrap();

    let result_path = test_dir_path.join("break-result.dot");
    std::fs::write(result_path, result.to_dot()).unwrap();

    assert_eq!(expected, result);
}

#[test]
#[ignore = "Figure out a better way to verify semantics"]
fn while_loop_with_continue() {
    let input = "
void test() {
    while (2) {
        int x = 0;
        continue;
    }

    return;
}
        ";
    let source = Source::new("test", input);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global = BasicBlock::initial(vec![]);

    let t0_var = Variable::new("__t_0", Type::I64);
    let x0_var = Variable::new("x_13661097461077092700", Type::I32);

    let func_initial_block = BasicBlock::new(vec![global.weak_ptr()], vec![]);

    let func_first_block = BasicBlock::new(vec![func_initial_block.weak_ptr()], vec![]);
    func_initial_block.add_statement(Statement::Jump(
        func_first_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let loop_cond_block = BasicBlock::new(
        vec![func_first_block.weak_ptr()],
        vec![Statement::Assignment {
            target: t0_var.clone(),
            value: Value::Constant(Constant::I64(2)),
        }],
    );
    func_first_block.add_statement(Statement::Jump(
        loop_cond_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    let func_end_block = BasicBlock::new(vec![], vec![Statement::Return(None)]);

    let loop_inner_block = BasicBlock::new(
        vec![loop_cond_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::SaveVariable { var: x0_var },
            Statement::Jump(loop_cond_block.clone(), ir::JumpMetadata::Linear),
            Statement::Jump(loop_cond_block.clone(), ir::JumpMetadata::Linear),
        ],
    );
    loop_cond_block.add_statement(Statement::JumpTrue(
        t0_var,
        loop_inner_block.clone(),
        ir::JumpMetadata::Linear,
    ));
    loop_cond_block.add_predecessor(loop_inner_block.weak_ptr());
    loop_cond_block.add_statement(Statement::Jump(
        func_end_block.clone(),
        ir::JumpMetadata::Linear,
    ));

    func_end_block.add_predecessor(loop_cond_block.weak_ptr());

    let expected = Program {
        global,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    dbg!(&expected);

    let test_file_path = {
        let raw = Path::new(file!());
        let mut result = PathBuf::new();
        for part in raw.components().skip(1) {
            result.push(part);
        }

        result
    };
    let test_dir_path = test_file_path.parent().unwrap();

    let result_path = test_dir_path.join("continue-result.dot");
    std::fs::write(result_path, result.to_dot()).unwrap();

    assert_eq!(expected, result);
}
