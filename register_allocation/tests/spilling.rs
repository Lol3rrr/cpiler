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
    fn spill_simple() {
        let test_program = "
int main() {
    int x = 0;
    int y = 3;
    int z = x + y;
    int w = x + y;
    return x + z + w + y;
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
        let tmp0 = ir::Variable::tmp(0, ir::Type::I32);
        let tmp1 = ir::Variable::tmp(1, ir::Type::I32);
        let y1 = y0.next_gen();
        let tmp2 = ir::Variable::tmp(2, ir::Type::I32);

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
                ir::Statement::SaveVariable { var: y0.clone() },
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
                    target: tmp0.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(x0.clone()),
                        right: ir::Operand::Variable(z0.clone()),
                    }),
                },
                ir::Statement::Assignment {
                    target: tmp1.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(tmp0),
                        right: ir::Operand::Variable(w0.clone()),
                    }),
                },
                ir::Statement::Assignment {
                    target: y1.clone(),
                    value: ir::Value::Unknown,
                },
                ir::Statement::Assignment {
                    target: tmp2.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(tmp1),
                        right: ir::Operand::Variable(y1.clone()),
                    }),
                },
                ir::Statement::Return(Some(tmp2)),
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
        let x0 = ir::Variable::new("x_12136962903682547977", ir::Type::I32);
        let t0 = ir::Variable::tmp(0, ir::Type::I64);
        let t1 = ir::Variable::tmp(1, ir::Type::I64);
        let t2 = ir::Variable::tmp(2, ir::Type::I64);
        let z1 = z0.next_gen();
        let t3 = ir::Variable::tmp(3, ir::Type::I64);
        let w1 = w0.next_gen();
        let t4 = ir::Variable::tmp(4, ir::Type::I64);
        let w2 = w1.next_gen();
        let z2 = z1.next_gen();
        let w3 = w2.next_gen();
        let t5 = ir::Variable::tmp(5, ir::Type::I32);
        let x1 = x0.next_gen();
        let t6 = ir::Variable::tmp(6, ir::Type::I32);

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
                    target: x0.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Constant(ir::Constant::I64(0)),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: x0.clone() },
                ir::Statement::Assignment {
                    target: t0.clone(),
                    value: ir::Value::Constant(ir::Constant::I64(1)),
                },
            ],
        );
        expected_ir_block1.add_statement(ir::Statement::Jump(expected_ir_block2.clone()));

        // True Block
        let expected_ir_block_true = ir::BasicBlock::new(
            vec![expected_ir_block2.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: t1.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Variable(z0.clone()),
                        target: ir::Type::I64,
                    }),
                },
                ir::Statement::Assignment {
                    target: t2.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(t1),
                        right: ir::Operand::Constant(ir::Constant::I64(6)),
                    }),
                },
                ir::Statement::Assignment {
                    target: z1.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Variable(t2),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: z1.clone() },
                ir::Statement::Assignment {
                    target: t3.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Constant(ir::Constant::I64(3)),
                        right: ir::Operand::Constant(ir::Constant::I64(5)),
                    }),
                },
                ir::Statement::Assignment {
                    target: w1.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Variable(t3),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: w1.clone() },
                ir::Statement::SaveVariable { var: x0.clone() },
            ],
        );
        expected_ir_block2
            .add_statement(ir::Statement::JumpTrue(t0, expected_ir_block_true.clone()));

        // False Block
        let expected_ir_block4 = ir::BasicBlock::new(
            vec![expected_ir_block2.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: t4.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Constant(ir::Constant::I64(2)),
                        right: ir::Operand::Constant(ir::Constant::I64(5)),
                    }),
                },
                ir::Statement::Assignment {
                    target: w2.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Variable(t4),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: w2.clone() },
                ir::Statement::SaveVariable { var: x0.clone() },
            ],
        );
        expected_ir_block2.add_statement(ir::Statement::Jump(expected_ir_block4.clone()));

        // After Block
        let expected_ir_block5 = ir::BasicBlock::new(
            vec![
                expected_ir_block_true.weak_ptr(),
                expected_ir_block4.weak_ptr(),
            ],
            vec![
                ir::Statement::Assignment {
                    target: z2.clone(),
                    value: ir::Value::Phi {
                        sources: vec![
                            ir::PhiEntry {
                                var: z1.clone(),
                                block: expected_ir_block_true.weak_ptr(),
                            },
                            ir::PhiEntry {
                                var: z0.clone(),
                                block: expected_ir_block4.weak_ptr(),
                            },
                        ],
                    },
                },
                ir::Statement::Assignment {
                    target: w3.clone(),
                    value: ir::Value::Phi {
                        sources: vec![
                            ir::PhiEntry {
                                var: w1.clone(),
                                block: expected_ir_block_true.weak_ptr(),
                            },
                            ir::PhiEntry {
                                var: w2.clone(),
                                block: expected_ir_block4.weak_ptr(),
                            },
                        ],
                    },
                },
                ir::Statement::Assignment {
                    target: t5.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(z2.clone()),
                        right: ir::Operand::Variable(w3.clone()),
                    }),
                },
                ir::Statement::Assignment {
                    target: x1.clone(),
                    value: ir::Value::Unknown,
                },
                ir::Statement::Assignment {
                    target: t6.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(t5),
                        right: ir::Operand::Variable(x1.clone()),
                    }),
                },
                ir::Statement::Return(Some(t6)),
            ],
        );
        expected_ir_block_true.add_statement(ir::Statement::Jump(expected_ir_block5.clone()));
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

        let z0 = ir::Variable::new("z_9670610037870495622", ir::Type::I32);
        let w0 = ir::Variable::new("w_9876566019052790868", ir::Type::I32);
        let tmp0 = ir::Variable::tmp(0, ir::Type::I64);
        let i1_0 = ir::Variable::new("i1_3138623224651233625", ir::Type::I32);
        let i2_0 = ir::Variable::new("i2_11341589265771357896", ir::Type::I32);
        let tmp1 = ir::Variable::tmp(1, ir::Type::I64);
        let tmp2 = ir::Variable::tmp(2, ir::Type::I64);
        let z1 = z0.next_gen();
        let w1 = w0.next_gen();
        let tmp3 = ir::Variable::tmp(3, ir::Type::I32);
        let tmp4 = ir::Variable::tmp(4, ir::Type::I32);
        let t1 = ir::Variable::new("t1_2388632724981900555", ir::Type::I32);
        let w2 = w1.next_gen();
        let w3 = w2.next_gen();
        let w4 = w3.next_gen();
        let tmp5 = ir::Variable::tmp(5, ir::Type::I64);
        let z2 = z1.next_gen();
        let z3 = z2.next_gen();
        let tmp6 = ir::Variable::tmp(6, ir::Type::I32);

        let expected_ir_block_start = ir::BasicBlock::new(vec![ir.global.weak_ptr()], vec![]);

        let expected_ir_block1 = ir::BasicBlock::new(
            vec![expected_ir_block_start.weak_ptr()],
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
                    target: tmp0.clone(),
                    value: ir::Value::Constant(ir::Constant::I64(1)),
                },
                ir::Statement::SaveVariable { var: z0.clone() },
                // JumpTrue tmp0
                // Jump
            ],
        );
        expected_ir_block_start.add_statement(ir::Statement::Jump(expected_ir_block1.clone()));

        let expected_ir_block_true = ir::BasicBlock::new(
            vec![expected_ir_block1.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: i1_0.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Constant(ir::Constant::I64(13)),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: i1_0.clone() },
                ir::Statement::Assignment {
                    target: i2_0.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Constant(ir::Constant::I64(23)),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: i2_0.clone() },
                ir::Statement::Assignment {
                    target: tmp1.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Variable(i1_0.clone()),
                        target: ir::Type::I64,
                    }),
                },
                ir::Statement::Assignment {
                    target: tmp2.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(tmp1),
                        right: ir::Operand::Constant(ir::Constant::I64(6)),
                    }),
                },
                ir::Statement::Assignment {
                    target: z1.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Variable(tmp2),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: z1.clone() },
                ir::Statement::Assignment {
                    target: w1.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(i2_0.clone()),
                        right: ir::Operand::Variable(i1_0.clone()),
                    }),
                },
                ir::Statement::SaveVariable { var: w1.clone() },
                ir::Statement::Assignment {
                    target: tmp3.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(z1.clone()),
                        right: ir::Operand::Variable(w1.clone()),
                    }),
                },
                ir::Statement::SaveVariable { var: w1.clone() },
                ir::Statement::Assignment {
                    target: tmp4.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(tmp3),
                        right: ir::Operand::Variable(i1_0.clone()),
                    }),
                },
                ir::Statement::Assignment {
                    target: t1.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Variable(tmp4),
                        right: ir::Operand::Variable(i2_0.clone()),
                    }),
                },
                ir::Statement::SaveVariable { var: t1.clone() },
                ir::Statement::Assignment {
                    target: w4.clone(),
                    value: ir::Value::Unknown,
                },
                // Jump to After-Block
            ],
        );
        expected_ir_block1.add_statement(ir::Statement::JumpTrue(
            tmp0,
            expected_ir_block_true.clone(),
        ));

        let expected_ir_block_false = ir::BasicBlock::new(
            vec![expected_ir_block1.weak_ptr()],
            vec![
                ir::Statement::Assignment {
                    target: tmp5.clone(),
                    value: ir::Value::Expression(ir::Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: ir::Operand::Constant(ir::Constant::I64(2)),
                        right: ir::Operand::Constant(ir::Constant::I64(5)),
                    }),
                },
                ir::Statement::Assignment {
                    target: w2.clone(),
                    value: ir::Value::Expression(ir::Expression::Cast {
                        base: ir::Operand::Variable(tmp5),
                        target: ir::Type::I32,
                    }),
                },
                ir::Statement::SaveVariable { var: w2.clone() },
                // Jump to After-Block
            ],
        );
        expected_ir_block1.add_statement(ir::Statement::Jump(expected_ir_block_false.clone()));

        let expected_ir_block_after = ir::BasicBlock::new(
            vec![
                expected_ir_block_true.weak_ptr(),
                expected_ir_block_false.weak_ptr(),
            ],
            vec![],
        );
        expected_ir_block_after.set_statements(vec![
            // TODO
            // This Part is not handled correctly as this load and Phi Node is not correct because
            // the z3 should instead be in the false block and then the Phi Node would be working
            // as normal
            ir::Statement::Assignment {
                target: z3.clone(),
                value: ir::Value::Unknown,
            },
            ir::Statement::Assignment {
                target: z2.clone(),
                value: ir::Value::Phi {
                    sources: vec![
                        ir::PhiEntry {
                            var: z1.clone(),
                            block: expected_ir_block_true.weak_ptr(),
                        },
                        ir::PhiEntry {
                            var: z3.clone(),
                            block: expected_ir_block_after.weak_ptr(),
                        },
                    ],
                },
            },
            // TODO
            // This is also not correct because w4 actually does not exist in the False Block and
            // therefore also cant be loaded from it
            ir::Statement::Assignment {
                target: w3.clone(),
                value: ir::Value::Phi {
                    sources: vec![
                        ir::PhiEntry {
                            var: w4.clone(),
                            block: expected_ir_block_true.weak_ptr(),
                        },
                        ir::PhiEntry {
                            var: w4.clone(),
                            block: expected_ir_block_false.weak_ptr(),
                        },
                    ],
                },
            },
            ir::Statement::Assignment {
                target: tmp6.clone(),
                value: ir::Value::Expression(ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                    left: ir::Operand::Variable(z2.clone()),
                    right: ir::Operand::Variable(w3.clone()),
                }),
            },
            ir::Statement::Return(Some(tmp6)),
        ]);
        expected_ir_block_true.add_statement(ir::Statement::Jump(expected_ir_block_after.clone()));
        expected_ir_block_false.add_statement(ir::Statement::Jump(expected_ir_block_after.clone()));

        let expected_ir_func = ir::FunctionDefinition {
            name: "main".to_string(),
            arguments: Vec::new(),
            return_ty: ir::Type::I32,
            block: expected_ir_block_start,
        };

        assert_eq!(expected_ir_func, main_func);
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
