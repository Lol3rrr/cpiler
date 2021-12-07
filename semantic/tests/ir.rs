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

        let initial_block = BasicBlock::new(vec![global_weak], vec![]);
        let initial_weak = Arc::downgrade(&initial_block);
        let inner_block = BasicBlock::new(
            vec![initial_weak],
            vec![
                Statement::Assignment {
                    target: Variable::new("x", 0, Type::I64),
                    value: Value::Constant(Constant::I64(13)),
                },
                Statement::Assignment {
                    target: Variable::new("y", 0, Type::I64),
                    value: Value::Expression(Expression::BinaryOp {
                        op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                        left: Operand::Variable(Variable::new("x", 0, Type::I64)),
                        right: Operand::Constant(Constant::I64(5)),
                    }),
                },
                Statement::Assignment {
                    target: Variable::new("x", 1, Type::I64),
                    value: Value::Constant(Constant::I64(12)),
                },
                Statement::Assignment {
                    target: Variable::new("y", 1, Type::I64),
                    value: Value::Expression(Expression::BinaryOp {
                        op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                        left: Operand::Variable(Variable::new("y", 0, Type::I64)),
                        right: Operand::Constant(Constant::I64(1)),
                    }),
                },
                Statement::Return(None),
            ],
        );
        initial_block.add_statement(Statement::Jump(inner_block));

        let expected = Program {
            global: global_block,
            functions: vec![FunctionDefinition {
                arguments: vec![],
                return_ty: Type::Void,
                block: initial_block,
            }],
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

        let func_initial_block = BasicBlock::new(
            vec![global_block.weak_ptr()],
            vec![Statement::Assignment {
                target: Variable::new("arg", 0, Type::I32),
                value: Value::Unknown,
            }],
        );

        let func_body_block = BasicBlock::new(
            vec![func_initial_block.weak_ptr()],
            vec![
                Statement::Assignment {
                    target: Variable::new("x", 0, Type::I32),
                    value: Value::Expression(Expression::Cast {
                        target: Type::I32,
                        base: Operand::Constant(Constant::I64(13)),
                    }),
                },
                Statement::Assignment {
                    target: Variable::new("y", 0, Type::I32),
                    value: Value::Expression(Expression::BinaryOp {
                        op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                        left: Operand::Variable(Variable::new("arg", 0, Type::I32)),
                        right: Operand::Variable(Variable::new("x", 0, Type::I32)),
                    }),
                },
                Statement::Return(None),
            ],
        );
        func_initial_block.add_statement(Statement::Jump(func_body_block));

        let expected = Program {
            global: global_block,
            functions: vec![FunctionDefinition {
                arguments: vec![("arg".to_string(), Type::I32)],
                return_ty: Type::Void,
                block: func_initial_block,
            }],
        };

        let result = aast.convert_to_ir();
        dbg!(&result);

        assert_eq!(expected, result);
    }
}

mod pointers {}

mod arrays {}

mod structs {}
