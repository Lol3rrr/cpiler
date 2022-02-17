#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum TestRegister {
    GeneralPurpose(u8),
    FloatingPoint(u8),
}

impl register_allocation::Register for TestRegister {
    fn reg_type(&self) -> register_allocation::RegisterType {
        match self {
            Self::GeneralPurpose(_) => register_allocation::RegisterType::GeneralPurpose,
            Self::FloatingPoint(_) => register_allocation::RegisterType::FloatingPoint,
        }
    }

    fn align_size(&self) -> (usize, usize) {
        match self {
            Self::GeneralPurpose(_) => (4, 4),
            Self::FloatingPoint(_) => (4, 4),
        }
    }
}

mod linear {

    use register_allocation::RegisterMapping;

    use super::*;

    #[test]
    fn no_spill() {
        let test_program = "
int main() {
    int x = 0;
    int y = x + 3;
    return y;
}
        ";
        let test_source = general::Source::new("test", test_program);
        let test_span: general::Span = test_source.clone().into();
        let tokens = tokenizer::tokenize(test_span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

        let expected_ir = semantic::parse(
            syntax::parse(tokenizer::tokenize(test_source.clone().into())).unwrap(),
        )
        .unwrap()
        .convert_to_ir(general::arch::Arch::X86_64);
        let expected_ir_func: ir::FunctionDefinition =
            expected_ir.functions.get("main").unwrap().clone();

        let main_func: ir::FunctionDefinition = ir.functions.get("main").unwrap().clone();

        let result_allocation = RegisterMapping::allocate(
            &main_func,
            &[
                TestRegister::GeneralPurpose(0),
                TestRegister::GeneralPurpose(1),
                TestRegister::GeneralPurpose(2),
            ],
        );
        dbg!(&result_allocation);

        dbg!(&main_func);

        assert_eq!(expected_ir_func, main_func);
    }

    #[test]
    fn spill() {
        let test_program = "
int main() {
    int x = 0;
    int y = 3;
    int z = x + y;
    int w = x + y;
    return x + z;
}
        ";
        let test_source = general::Source::new("test", test_program);
        let test_span: general::Span = test_source.clone().into();
        let tokens = tokenizer::tokenize(test_span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

        dbg!(&ir);

        let main_func: ir::FunctionDefinition = ir.functions.get("main").unwrap().clone();

        let result_allocation = RegisterMapping::allocate(
            &main_func,
            &[
                TestRegister::GeneralPurpose(0),
                TestRegister::GeneralPurpose(1),
                TestRegister::GeneralPurpose(2),
            ],
        );
        dbg!(&result_allocation);

        dbg!(&main_func);

        let expected_ir_block1 = ir::BasicBlock::new(vec![ir.global.weak_ptr()], vec![]);

        let x0 = ir::Variable::new("x_15892656085374368478", ir::Type::I32);
        let y0 = ir::Variable::new("y_16559376576152361384", ir::Type::I32);
        let z0 = ir::Variable::new("z_11118331272375206556", ir::Type::I32);
        let w0 = ir::Variable::new("w_13930978485168054992", ir::Type::I32);
        let z1 = z0.next_gen();
        let t0 = ir::Variable::tmp(0, ir::Type::I32);

        let expected_ir_block2 = ir::BasicBlock::new(
            vec![expected_ir_block1.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: x0.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Constant(ir::Constant::I64(0)),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: x0.clone() },
                ir::Statement::Assignment {
                    target: y0.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Constant(ir::Constant::I64(3)),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: y0.clone() },
                ir::Statement::Assignment {
                    target: z0.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(x0.clone()),
                        right: ir::Operand::Variable(y0.clone()),
                    }),
                },
                ir::Statement::SaveVariable { var: z0.clone() },
                ir::Statement::SaveVariable { var: z0.clone() },
                ir::Statement::Assignment {
                    target: w0.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(x0.clone()),
                        right: ir::Operand::Variable(y0.clone()),
                    }),
                },
                ir::Statement::SaveVariable { var: w0.clone() },
                ir::Statement::Assignment {
                    target: z1.clone(),
                    value: ir::Value::Unknown,
                },
                ir::Statement::Assignment {
                    target: t0.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(x0.clone()),
                        right: ir::Operand::Variable(z1.clone()),
                    }),
                },
                ir::Statement::Return(Some(t0.clone())),
            ],
        );
        expected_ir_block1.add_statement(ir::Statement::Jump(expected_ir_block2));

        let expected_ir_func = ir::FunctionDefinition {
            name: "main".to_string(),
            arguments: Vec::new(),
            return_ty: ir::Type::I32,
            block: expected_ir_block1,
        };

        assert_eq!(expected_ir_func, main_func);
    }

    #[test]
    fn spill_with_cond_later() {
        let test_program = "
int main() {
    int x = 0;
    int y = 3;
    int z = x + y;
    int w = x + y;
    int res = x + z;
    
    if (1) {
        int x = 13;
    } else {
        int x = 25;
    }

    return res;
}
        ";
        let test_source = general::Source::new("test", test_program);
        let test_span: general::Span = test_source.clone().into();
        let tokens = tokenizer::tokenize(test_span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

        dbg!(&ir);

        let main_func: ir::FunctionDefinition = ir.functions.get("main").unwrap().clone();

        let result_allocation = RegisterMapping::allocate(
            &main_func,
            &[
                TestRegister::GeneralPurpose(0),
                TestRegister::GeneralPurpose(1),
                TestRegister::GeneralPurpose(2),
            ],
        );
        dbg!(&result_allocation);

        dbg!(&main_func);

        let expected_ir_block1 = ir::BasicBlock::new(vec![ir.global.weak_ptr()], vec![]);

        let x0 = ir::Variable::new("x_15892656085374368478", ir::Type::I32);
        let y0 = ir::Variable::new("y_16559376576152361384", ir::Type::I32);
        let z0 = ir::Variable::new("z_11118331272375206556", ir::Type::I32);
        let w0 = ir::Variable::new("w_13930978485168054992", ir::Type::I32);
        let z1 = z0.next_gen();
        let res0 = ir::Variable::new("res_18266311551537244604", ir::Type::I32);
        let t0 = ir::Variable::tmp(0, ir::Type::I64);
        let x0_1 = ir::Variable::new("x_12489501789731851193", ir::Type::I32);
        let x0_2 = ir::Variable::new("x_7983228676338287122", ir::Type::I32);
        let t1 = ir::Variable::tmp(1, ir::Type::I32);

        let expected_ir_block2 = ir::BasicBlock::new(
            vec![expected_ir_block1.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: x0.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Constant(ir::Constant::I64(0)),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: x0.clone() },
                ir::Statement::Assignment {
                    target: y0.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Constant(ir::Constant::I64(3)),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: y0.clone() },
                ir::Statement::Assignment {
                    target: z0.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(x0.clone()),
                        right: ir::Operand::Variable(y0.clone()),
                    }),
                },
                ir::Statement::SaveVariable { var: z0.clone() },
                ir::Statement::SaveVariable { var: z0.clone() },
                ir::Statement::Assignment {
                    target: w0.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(x0.clone()),
                        right: ir::Operand::Variable(y0.clone()),
                    }),
                },
                ir::Statement::SaveVariable { var: w0.clone() },
                ir::Statement::Assignment {
                    target: z1.clone(),
                    value: ir::Value::Unknown,
                },
                ir::Statement::Assignment {
                    target: res0.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(x0.clone()),
                        right: ir::Operand::Variable(z1.clone()),
                    }),
                },
                ir::Statement::SaveVariable { var: res0.clone() },
                ir::Statement::Assignment {
                    target: t0.clone(),
                    value: ir::Value::Constant(ir::Constant::I64(1)),
                },
            ],
        );
        expected_ir_block1.add_statement(ir::Statement::Jump(expected_ir_block2.clone()));

        // True Block
        let expected_ir_block4 = ir::BasicBlock::new(
            vec![expected_ir_block2.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: x0_2.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Constant(ir::Constant::I64(13)),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: x0_2.clone() },
            ],
        );
        expected_ir_block2.add_statement(ir::Statement::JumpTrue(
            t0.clone(),
            expected_ir_block4.clone(),
        ));

        // False Block
        let expected_ir_block3 = ir::BasicBlock::new(
            vec![expected_ir_block2.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: x0_1.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Constant(ir::Constant::I64(25)),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: x0_1.clone() },
            ],
        );
        expected_ir_block2.add_statement(ir::Statement::Jump(expected_ir_block3.clone()));

        // After Block
        let expected_ir_block5 = ir::BasicBlock::new(
            vec![expected_ir_block3.weak_ptr(), expected_ir_block4.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: t1.clone(),
                    value: ir::Value::Variable(res0.clone()),
                },
                ir::Statement::Return(Some(t1.clone())),
            ],
        );
        expected_ir_block3.add_statement(ir::Statement::Jump(expected_ir_block5.clone()));
        expected_ir_block4.add_statement(ir::Statement::Jump(expected_ir_block5.clone()));

        let expected_ir_func = ir::FunctionDefinition {
            name: "main".to_string(),
            arguments: Vec::new(),
            return_ty: ir::Type::I32,
            block: expected_ir_block1,
        };

        assert_eq!(expected_ir_func, main_func);
    }
}

mod conditional {
    use register_allocation::RegisterMapping;

    use super::*;

    #[test]
    fn spill_outer() {
        let test_program = "
int main() {
    int z = 0;
    int w = 0;
    int x = 0;

    if (1) {
        z = z + 6;
        w = 3 + 5;
    } else {
        w = 2 + 5;
    }

    return z + w + x;
}
        ";
        let test_source = general::Source::new("test", test_program);
        let test_span: general::Span = test_source.clone().into();
        let tokens = tokenizer::tokenize(test_span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

        dbg!(&ir);

        let main_func: ir::FunctionDefinition = ir.functions.get("main").unwrap().clone();

        let result_allocation = RegisterMapping::allocate(
            &main_func,
            &[
                TestRegister::GeneralPurpose(0),
                TestRegister::GeneralPurpose(1),
                TestRegister::GeneralPurpose(2),
                TestRegister::GeneralPurpose(3),
                TestRegister::GeneralPurpose(4),
            ],
        );
        dbg!(&result_allocation);

        dbg!(&main_func);

        let z0 = ir::Variable::new("z_9670610037870495622", ir::Type::I32);
        let w0 = ir::Variable::new("w_9876566019052790868", ir::Type::I32);
        let t0 = ir::Variable::tmp(0, ir::Type::I64);
        let t1 = ir::Variable::tmp(1, ir::Type::I64);
        let z1 = z0.next_gen();
        let t2 = ir::Variable::tmp(2, ir::Type::I64);
        let w1 = w0.next_gen();
        let t3 = ir::Variable::tmp(3, ir::Type::I64);
        let w2 = w1.next_gen();

        let expected_ir_block1 = ir::BasicBlock::new(vec![ir.global.weak_ptr()], vec![]);

        let expected_ir_block2 = ir::BasicBlock::new(
            vec![expected_ir_block1.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: z0.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Constant(ir::Constant::I64(0)),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: z0.clone() },
                ir::Statement::Assignment {
                    target: w0.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Constant(ir::Constant::I64(0)),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: w0.clone() },
                ir::Statement::Assignment {
                    target: t0.clone(),
                    value: ir::Value::Constant(ir::Constant::I64(0)),
                },
            ],
        );
        expected_ir_block1.add_statement(ir::Statement::Jump(expected_ir_block2.clone()));

        // True Block
        let expected_ir_block3 = ir::BasicBlock::new(
            vec![expected_ir_block2.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: t1.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Constant(ir::Constant::I64(2)),
                        right: ir::Operand::Constant(ir::Constant::I64(6)),
                    }),
                },
                ir::Statement::Assignment {
                    target: z1.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Variable(t1),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: z1.clone() },
                ir::Statement::Assignment {
                    target: t2.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Constant(ir::Constant::I64(3)),
                        right: ir::Operand::Constant(ir::Constant::I64(5)),
                    }),
                },
                ir::Statement::Assignment {
                    target: w1.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Variable(t2),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: w1.clone() },
            ],
        );
        expected_ir_block2.add_statement(ir::Statement::JumpTrue(t0, expected_ir_block3.clone()));

        // False Block
        let expected_ir_block4 = ir::BasicBlock::new(
            vec![expected_ir_block2.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: t3.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Constant(ir::Constant::I64(2)),
                        right: ir::Operand::Constant(ir::Constant::I64(5)),
                    }),
                },
                ir::Statement::Assignment {
                    target: w2.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Variable(t3),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: w2.clone() },
            ],
        );
        expected_ir_block2.add_statement(ir::Statement::Jump(expected_ir_block4.clone()));

        // After Block
        let expected_ir_block5 = ir::BasicBlock::new(
            vec![expected_ir_block3.weak_ptr(), expected_ir_block4.weak_ptr()],
            vec![],
        );
        expected_ir_block3.add_statement(ir::Statement::Jump(expected_ir_block5.clone()));
        expected_ir_block4.add_statement(ir::Statement::Jump(expected_ir_block5.clone()));

        let expected_ir_func = ir::FunctionDefinition {
            name: "main".to_string(),
            arguments: Vec::new(),
            return_ty: ir::Type::I32,
            block: expected_ir_block1,
        };

        assert_eq!(expected_ir_func, main_func);
    }

    /// This Test enters an infinite loop sort of thing and i dont know why
    #[test]
    fn spill_inner() {
        let test_program = "
int main() {
    int z = 0;
    int w = 0;

    if (1) {
        int i1 = 13;
        int i2 = 23;
        z = i1 + 6;
        w = i2 + i1;
        int t1 = z + w + i1 + i2;
    } else {
        w = 2 + 5;
    }

    return z + w;
}
        ";
        let test_source = general::Source::new("test", test_program);
        let test_span: general::Span = test_source.clone().into();
        let tokens = tokenizer::tokenize(test_span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

        dbg!(&ir);

        let main_func: ir::FunctionDefinition = ir.functions.get("main").unwrap().clone();

        let result_allocation = RegisterMapping::allocate(
            &main_func,
            &[
                TestRegister::GeneralPurpose(0),
                TestRegister::GeneralPurpose(1),
                TestRegister::GeneralPurpose(2),
                TestRegister::GeneralPurpose(3),
                TestRegister::GeneralPurpose(4),
            ],
        );
        dbg!(&result_allocation);

        dbg!(&main_func);

        // TODO
    }
}

mod loops {
    use register_allocation::RegisterMapping;

    use super::*;

    #[test]
    fn spill_outer() {
        let test_program = "
int main() {
    int z = 0;
    int w = 0;
    int tmp = 13;

    while (1) {
        z = 2 + 6;
        w = 5 + 3;
    }

    return z + w + tmp;
}
        ";
        let test_source = general::Source::new("test", test_program);
        let test_span: general::Span = test_source.clone().into();
        let tokens = tokenizer::tokenize(test_span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

        dbg!(&ir);

        let main_func: ir::FunctionDefinition = ir.functions.get("main").unwrap().clone();

        let result_allocation = RegisterMapping::allocate(
            &main_func,
            &[
                TestRegister::GeneralPurpose(0),
                TestRegister::GeneralPurpose(1),
                TestRegister::GeneralPurpose(2),
                TestRegister::GeneralPurpose(3),
                TestRegister::GeneralPurpose(4),
            ],
        );
        dbg!(&result_allocation);

        dbg!(&main_func);

        // TODO
    }

    #[test]
    fn spill_inner() {
        let test_program = "
int main() {
    int z = 0;
    int w = 0;

    while (1) {
        z = 2 + 6;
        w = 3 + 7;
    }

    return z + w;
}
        ";
        let test_source = general::Source::new("test", test_program);
        let test_span: general::Span = test_source.clone().into();
        let tokens = tokenizer::tokenize(test_span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

        dbg!(&ir);

        let main_func: ir::FunctionDefinition = ir.functions.get("main").unwrap().clone();

        let result_allocation = RegisterMapping::allocate(
            &main_func,
            &[
                TestRegister::GeneralPurpose(0),
                TestRegister::GeneralPurpose(1),
                TestRegister::GeneralPurpose(2),
                TestRegister::GeneralPurpose(3),
            ],
        );
        dbg!(&result_allocation);

        dbg!(&main_func);

        // TODO
    }
}
