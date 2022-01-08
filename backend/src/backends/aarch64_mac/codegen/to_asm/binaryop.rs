use crate::backends::aarch64_mac::{
    asm,
    codegen::{util, Context},
    ArmRegister,
};

pub fn to_asm(
    op: ir::BinaryOp,
    t_reg: asm::GPRegister,
    left: ir::Operand,
    right: ir::Operand,
    ctx: &Context,
    instructions: &mut Vec<asm::Instruction>,
) {
    match op {
        ir::BinaryOp::Logic(log_op) => {
            match (&left, &right) {
                (ir::Operand::Variable(var), ir::Operand::Constant(con)) => {
                    let (var_reg, cmp_reg) = match ctx.registers.get(&var).unwrap() {
                        ArmRegister::GeneralPurpose(n) => {
                            (asm::GPRegister::DWord(*n), asm::GPRegister::DWord(9))
                        }
                        ArmRegister::FloatingPoint(n) => {
                            panic!("Not yet supported")
                        }
                    };

                    let store_cond_instr = util::constant_to_asm(&con, cmp_reg.clone());

                    instructions.extend(store_cond_instr);

                    instructions.push(asm::Instruction::CmpShifted {
                        first: var_reg,
                        second: cmp_reg,
                        shift: asm::Shift::LSL,
                        amount: 0,
                    });
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            };

            let condition = match log_op {
                ir::BinaryLogicOp::Greater => {
                    if left.ty().signed() {
                        asm::Cond::Gt
                    } else {
                        todo!("Unsigned Greater than comparison")
                    }
                }
                ir::BinaryLogicOp::Less => {
                    if left.ty().signed() {
                        asm::Cond::Lt
                    } else {
                        panic!("Unsigned Less than comparison")
                    }
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            };

            instructions.push(asm::Instruction::CSet {
                target: t_reg,
                condition,
            });
        }
        ir::BinaryOp::Arith(arith_op) => {
            let (first_reg, second_reg) = match (left, right) {
                (ir::Operand::Variable(var), ir::Operand::Constant(con)) => {
                    let (var_reg, imm_reg) = match ctx.registers.get(&var).unwrap() {
                        ArmRegister::GeneralPurpose(n) => {
                            (asm::GPRegister::DWord(*n), asm::GPRegister::DWord(9))
                        }
                        ArmRegister::FloatingPoint(n) => {
                            panic!("Not yet supported")
                        }
                    };

                    let imm_store_instr = util::constant_to_asm(&con, imm_reg.clone());
                    instructions.extend(imm_store_instr);

                    (var_reg, imm_reg)
                }
                (ir::Operand::Variable(first_var), ir::Operand::Variable(second_var)) => {
                    let first_var_reg = match ctx.registers.get(&first_var).unwrap() {
                        ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                        ArmRegister::FloatingPoint(n) => {
                            panic!("Not yet supported")
                        }
                    };

                    let second_var_reg = match ctx.registers.get(&second_var).unwrap() {
                        ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                        ArmRegister::FloatingPoint(n) => {
                            panic!("Not yet supported")
                        }
                    };

                    (first_var_reg, second_var_reg)
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            };

            match arith_op {
                ir::BinaryArithmeticOp::Add => {
                    instructions.push(asm::Instruction::AddRegisterShifted {
                        dest: t_reg,
                        src1: first_reg,
                        src2: second_reg,
                        shift: asm::Shift::LSL,
                        amount: 0,
                    });
                }
                ir::BinaryArithmeticOp::Sub => {
                    instructions.push(asm::Instruction::SubRegisterShifted {
                        dest: t_reg,
                        src1: first_reg,
                        src2: second_reg,
                        shift: asm::Shift::LSL,
                        amount: 0,
                    });
                }
                ir::BinaryArithmeticOp::Multiply => {
                    instructions.push(asm::Instruction::MultiplyRegister {
                        dest: t_reg,
                        src1: first_reg,
                        src2: second_reg,
                    });
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            };
        }
        other => {
            dbg!(&other);
            todo!()
        }
    };
}
