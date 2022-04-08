use register_allocation::{AllocationCtx, RegisterMapping};

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

    let expected_ir =
        semantic::parse(syntax::parse(tokenizer::tokenize(test_source.into())).unwrap())
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
        AllocationCtx { build_path: None },
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
    let test_span: general::Span = test_source.into();
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
        AllocationCtx { build_path: None },
    );
    dbg!(&result_allocation);

    dbg!(&main_func);

    let expected_ir_block1 = ir::BasicBlock::new(vec![ir.global.weak_ptr()], vec![]);

    let x0 = ir::Variable::new("x_15892656085374368478", ir::Type::I32);
    let y0 = ir::Variable::new("y_16559376576152361384", ir::Type::I32);
    let z0 = ir::Variable::new("z_11118331272375206556", ir::Type::I32);
    let w0 = ir::Variable::new("w_13930978485168054992", ir::Type::I32);
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
                    right: ir::Operand::Variable(y0),
                }),
            },
            ir::Statement::SaveVariable { var: w0.clone() },
            ir::Statement::Assignment {
                target: tmp0.clone(),
                value: ir::Value::Expression(ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                    left: ir::Operand::Variable(x0),
                    right: ir::Operand::Variable(z0),
                }),
            },
            ir::Statement::Assignment {
                target: tmp1.clone(),
                value: ir::Value::Expression(ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                    left: ir::Operand::Variable(tmp0),
                    right: ir::Operand::Variable(w0),
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
                    right: ir::Operand::Variable(y1),
                }),
            },
            ir::Statement::Return(Some(tmp2)),
        ],
    );
    expected_ir_block1.add_statement(ir::Statement::Jump(
        expected_ir_block2,
        ir::JumpMetadata::Linear,
    ));

    let expected_ir_func = ir::FunctionDefinition {
        name: "main".to_string(),
        arguments: Vec::new(),
        return_ty: ir::Type::I32,
        block: expected_ir_block1,
    };

    // TODO
    // assert_eq!(expected_ir_func, main_func);
}

#[test]
fn spill_with_cond_later() {
    let test_program = "
int main() {
    int x = 0;
    int y = 3;
    int z = x + y;
    int w = x + y;
    int res = x + z + w + y;
    
    if (1) {
        int x = 13;
    } else {
        int x = 25;
    }

    return res;
}
        ";
    let test_source = general::Source::new("test", test_program);
    let test_span: general::Span = test_source.into();
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
        AllocationCtx { build_path: None },
    );
    dbg!(&result_allocation);

    dbg!(&main_func);

    let expected_ir_block1 = ir::BasicBlock::new(vec![ir.global.weak_ptr()], vec![]);

    let x0 = ir::Variable::new("x_15892656085374368478", ir::Type::I32);
    let y0 = ir::Variable::new("y_16559376576152361384", ir::Type::I32);
    let z0 = ir::Variable::new("z_11118331272375206556", ir::Type::I32);
    let w0 = ir::Variable::new("w_13930978485168054992", ir::Type::I32);
    let tmp0 = ir::Variable::tmp(0, ir::Type::I32);
    let tmp1 = ir::Variable::tmp(1, ir::Type::I32);
    let y1 = y0.next_gen();
    let res0 = ir::Variable::new("res_18266311551537244604", ir::Type::I32);
    let tmp2 = ir::Variable::tmp(2, ir::Type::I64);
    let x0_true = ir::Variable::new("x_13517971419422033678", ir::Type::I32);
    let x0_false = ir::Variable::new("x_10778296925075244503", ir::Type::I32);
    let tmp3 = ir::Variable::tmp(3, ir::Type::I32);

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
                    right: ir::Operand::Variable(y0),
                }),
            },
            ir::Statement::SaveVariable { var: w0.clone() },
            ir::Statement::Assignment {
                target: tmp0.clone(),
                value: ir::Value::Expression(ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                    left: ir::Operand::Variable(x0),
                    right: ir::Operand::Variable(z0),
                }),
            },
            ir::Statement::Assignment {
                target: tmp1.clone(),
                value: ir::Value::Expression(ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                    left: ir::Operand::Variable(tmp0),
                    right: ir::Operand::Variable(w0),
                }),
            },
            ir::Statement::Assignment {
                target: y1.clone(),
                value: ir::Value::Unknown,
            },
            ir::Statement::Assignment {
                target: res0.clone(),
                value: ir::Value::Expression(ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                    left: ir::Operand::Variable(tmp1),
                    right: ir::Operand::Variable(y1),
                }),
            },
            ir::Statement::SaveVariable { var: res0.clone() },
            ir::Statement::Assignment {
                target: tmp2.clone(),
                value: ir::Value::Constant(ir::Constant::I64(1)),
            },
        ],
    );
    expected_ir_block1.add_statement(ir::Statement::Jump(
        expected_ir_block2.clone(),
        ir::JumpMetadata::Linear,
    ));

    // True Block
    let expected_ir_block4 = ir::BasicBlock::new(
        vec![expected_ir_block2.weak_ptr()],
        vec![
            ir::Statement::Assignment {
                target: x0_true.clone(),
                value: ir::Value::Expression(ir::Expression::Cast {
                    base: ir::Operand::Constant(ir::Constant::I64(13)),
                    target: ir::Type::I32,
                }),
            },
            ir::Statement::SaveVariable { var: x0_true },
        ],
    );
    expected_ir_block2.add_statement(ir::Statement::JumpTrue(
        tmp2,
        expected_ir_block4.clone(),
        ir::JumpMetadata::Linear,
    ));

    // False Block
    let expected_ir_block3 = ir::BasicBlock::new(
        vec![expected_ir_block2.weak_ptr()],
        vec![
            ir::Statement::Assignment {
                target: x0_false.clone(),
                value: ir::Value::Expression(ir::Expression::Cast {
                    base: ir::Operand::Constant(ir::Constant::I64(25)),
                    target: ir::Type::I32,
                }),
            },
            ir::Statement::SaveVariable { var: x0_false },
        ],
    );
    expected_ir_block2.add_statement(ir::Statement::Jump(
        expected_ir_block3.clone(),
        ir::JumpMetadata::Linear,
    ));

    // After Block
    let expected_ir_block5 = ir::BasicBlock::new(
        vec![expected_ir_block3.weak_ptr(), expected_ir_block4.weak_ptr()],
        vec![
            ir::Statement::Assignment {
                target: tmp3.clone(),
                value: ir::Value::Variable(res0),
            },
            ir::Statement::Return(Some(tmp3)),
        ],
    );
    expected_ir_block3.add_statement(ir::Statement::Jump(
        expected_ir_block5.clone(),
        ir::JumpMetadata::Linear,
    ));
    expected_ir_block4.add_statement(ir::Statement::Jump(
        expected_ir_block5,
        ir::JumpMetadata::Linear,
    ));

    let expected_ir_func = ir::FunctionDefinition {
        name: "main".to_string(),
        arguments: Vec::new(),
        return_ty: ir::Type::I32,
        block: expected_ir_block1,
    };

    // TODO
    // assert_eq!(expected_ir_func, main_func);
}
