mod scalar {
    use std::sync::Arc;

    use general::{Source, Span};
    use ir::{
        BasicBlock, BinaryArithmeticOp, BinaryOp, Constant, Expression, FunctionDefinition,
        Operand, Program, Statement, Type, Value, Variable,
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
        let span: Span = source.clone().into();
        let tokens = tokenizer::tokenize(span);
        let syntax_ast = syntax::parse(tokens).unwrap();
        let input = semantic::parse(syntax_ast).unwrap();

        let global_block = BasicBlock::initial(vec![]);
        let global_weak = Arc::downgrade(&global_block);

        let x_var = Variable::new("x", Type::I64);
        let x1_var = x_var.next_gen();
        let y_var = Variable::new("y", Type::I64);
        let y1_var = y_var.next_gen();

        let initial_block = BasicBlock::new(vec![global_weak], vec![]);
        let initial_weak = Arc::downgrade(&initial_block);
        let inner_block = BasicBlock::new(
            vec![initial_weak],
            vec![
                Statement::Assignment {
                    target: x_var.clone(),
                    value: Value::Constant(Constant::I64(13)),
                },
                Statement::Assignment {
                    target: y_var.clone(),
                    value: Value::Expression(Expression::BinaryOp {
                        op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                        left: Operand::Variable(x_var.clone()),
                        right: Operand::Constant(Constant::I64(5)),
                    }),
                },
                Statement::Assignment {
                    target: x1_var.clone(),
                    value: Value::Constant(Constant::I64(12)),
                },
                Statement::Assignment {
                    target: y1_var.clone(),
                    value: Value::Expression(Expression::BinaryOp {
                        op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                        left: Operand::Variable(y_var.clone()),
                        right: Operand::Constant(Constant::I64(1)),
                    }),
                },
                Statement::Return(None),
            ],
        );
        initial_block.add_statement(Statement::Jump(inner_block));

        let expected = Program {
            global: global_block,
            functions: vec![(
                "test".to_string(),
                FunctionDefinition {
                    arguments: vec![],
                    return_ty: Type::Void,
                    block: initial_block,
                },
            )]
            .into_iter()
            .collect(),
        };

        let result = input.convert_to_ir();
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
        let span: Span = source.clone().into();
        let tokens = tokenizer::tokenize(span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();

        let global_block = BasicBlock::initial(vec![]);

        let arg_var = Variable::new("arg", Type::I32);
        let x_var = Variable::new("x", Type::I32);
        let y_var = Variable::new("y", Type::I32);

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
                Statement::Assignment {
                    target: y_var.clone(),
                    value: Value::Expression(Expression::BinaryOp {
                        op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                        left: Operand::Variable(arg_var.clone()),
                        right: Operand::Variable(x_var.clone()),
                    }),
                },
                Statement::Return(None),
            ],
        );
        func_initial_block.add_statement(Statement::Jump(func_body_block));

        let expected = Program {
            global: global_block,
            functions: vec![(
                "test".to_string(),
                FunctionDefinition {
                    arguments: vec![("arg".to_string(), Type::I32)],
                    return_ty: Type::Void,
                    block: func_initial_block,
                },
            )]
            .into_iter()
            .collect(),
        };

        let result = aast.convert_to_ir();
        dbg!(&result);

        assert_eq!(expected, result);
    }
}

mod ifs {
    use general::{Source, Span};
    use ir::{
        BasicBlock, Constant, Expression, FunctionDefinition, Operand, PhiEntry, Program,
        Statement, Type, Value, Variable,
    };

    #[test]
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
        let span: Span = source.clone().into();
        let tokens = tokenizer::tokenize(span);
        let syntax_ast = syntax::parse(tokens).unwrap();
        let input = semantic::parse(syntax_ast).unwrap();

        let global_block = BasicBlock::initial(vec![]);

        let x_var = Variable::new("x", Type::I32);
        let t0_64_var = Variable::new("__t_0", Type::I64);
        let t0_32_var = Variable::new("__t_0", Type::I32);
        let x1_var = x_var.next_gen();
        let y_var = Variable::new("y", Type::I32);

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
                Statement::Assignment {
                    target: t0_64_var.clone(),
                    value: Value::Constant(Constant::I64(2)),
                },
            ],
        );
        function_arg_block.add_statement(Statement::Jump(function_first_block.clone()));

        let inner_if_block = BasicBlock::new(
            vec![function_first_block.weak_ptr()],
            vec![Statement::Assignment {
                target: x1_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(2)),
                }),
            }],
        );
        function_first_block.add_statement(Statement::JumpTrue(
            t0_64_var.clone(),
            inner_if_block.clone(),
        ));

        let end_block = BasicBlock::new(
            vec![inner_if_block.weak_ptr(), function_first_block.weak_ptr()],
            vec![
                Statement::Assignment {
                    target: t0_32_var.clone(),
                    value: Value::Phi {
                        sources: vec![
                            PhiEntry {
                                block: function_first_block.weak_ptr(),
                                var: x1_var.clone(),
                            },
                            PhiEntry {
                                block: inner_if_block.weak_ptr(),
                                var: x_var.clone(),
                            },
                        ],
                    },
                },
                Statement::Assignment {
                    target: y_var.clone(),
                    value: Value::Variable(t0_32_var.clone()),
                },
                Statement::Return(None),
            ],
        );
        function_first_block.add_statement(Statement::Jump(end_block.clone()));
        inner_if_block.add_statement(Statement::Jump(end_block.clone()));

        let expected = Program {
            global: global_block,
            functions: vec![(
                "test".to_string(),
                FunctionDefinition {
                    arguments: vec![],
                    return_ty: Type::Void,
                    block: function_arg_block,
                },
            )]
            .into_iter()
            .collect(),
        };

        let result = input.convert_to_ir();
        dbg!(&result);

        std::fs::write("./result.dot", result.to_dot());
        std::fs::write("./expected.dot", expected.to_dot());

        assert_eq!(expected, result);
    }

    #[test]
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
        let span: Span = source.clone().into();
        let tokens = tokenizer::tokenize(span);
        let syntax_ast = syntax::parse(tokens).unwrap();
        let input = semantic::parse(syntax_ast).unwrap();

        let global_block = BasicBlock::initial(vec![]);

        let function_start_block = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

        let x0_var = Variable::new("x", Type::I32);
        let x1_var = x0_var.next_gen();
        let x2_var = x1_var.next_gen();
        let t0_var = Variable::new("__t_0", Type::I64);
        let t0_phi_var = Variable::new("__t_0", Type::I32);
        let y_var = Variable::new("y", Type::I32);

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
                Statement::Assignment {
                    target: t0_var.clone(),
                    value: Value::Constant(Constant::I64(2)),
                },
            ],
        );
        function_start_block.add_statement(Statement::Jump(function_first.clone()));

        let true_block = BasicBlock::new(
            vec![function_first.weak_ptr()],
            vec![Statement::Assignment {
                target: x1_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(2)),
                }),
            }],
        );
        function_first.add_statement(Statement::JumpTrue(t0_var.clone(), true_block.clone()));

        let false_block = BasicBlock::new(
            vec![function_first.weak_ptr()],
            vec![Statement::Assignment {
                target: x2_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(3)),
                }),
            }],
        );
        function_first.add_statement(Statement::Jump(false_block.clone()));

        let function_second = BasicBlock::new(
            vec![true_block.weak_ptr(), false_block.weak_ptr()],
            vec![
                Statement::Assignment {
                    target: t0_phi_var.clone(),
                    value: Value::Phi {
                        sources: vec![
                            PhiEntry {
                                block: true_block.weak_ptr(),
                                var: x1_var.clone(),
                            },
                            PhiEntry {
                                block: false_block.weak_ptr(),
                                var: x2_var.clone(),
                            },
                        ],
                    },
                },
                Statement::Assignment {
                    target: y_var.clone(),
                    value: Value::Variable(t0_phi_var.clone()),
                },
                Statement::Return(None),
            ],
        );

        true_block.add_statement(Statement::Jump(function_second.clone()));
        false_block.add_statement(Statement::Jump(function_second.clone()));

        let expected = Program {
            global: global_block,
            functions: vec![(
                "test".to_string(),
                FunctionDefinition {
                    arguments: vec![],
                    return_ty: Type::Void,
                    block: function_start_block,
                },
            )]
            .into_iter()
            .collect(),
        };

        let result = input.convert_to_ir();
        dbg!(&result);

        std::fs::write("./result.dot", result.to_dot());
        std::fs::write("./expected.dot", expected.to_dot());

        assert_eq!(expected, result);
    }
}

mod pointers {}

mod arrays {}

mod structs {}
