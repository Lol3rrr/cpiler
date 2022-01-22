use crate::backends::aarch64_mac::{asm, codegen::Context};

pub fn to_asm(
    op: ir::UnaryOp,
    t_reg: asm::Register,
    base: ir::Operand,
    ctx: &Context,
    instr: &mut Vec<asm::Instruction>,
) {
    let base_reg = match base {
        ir::Operand::Variable(base_var) => match ctx.registers.get_reg(&base_var).unwrap() {
            asm::Register::GeneralPurpose(gp) => match gp {
                asm::GPRegister::DWord(n) => asm::GPRegister::DWord(n),
                asm::GPRegister::Word(n) => asm::GPRegister::Word(n),
            },
            asm::Register::FloatingPoint(_) => {
                todo!("Floating Point Registers")
            }
        },
        ir::Operand::Constant(base_con) => match base_con {
            ir::Constant::I64(val) => {
                let base_reg = match &t_reg {
                    asm::Register::GeneralPurpose(gp) => match gp {
                        asm::GPRegister::DWord(_) => asm::GPRegister::DWord(9),
                        asm::GPRegister::Word(_) => asm::GPRegister::Word(9),
                    },
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                };

                instr.push(asm::Instruction::MovI64 {
                    dest: base_reg.clone(),
                    value: val,
                });

                base_reg
            }
            other => {
                dbg!(&other);
                todo!()
            }
        },
    };

    match (t_reg, op) {
        (asm::Register::GeneralPurpose(t_reg), ir::UnaryOp::Arith(arith_op)) => {
            match arith_op {
                ir::UnaryArithmeticOp::Increment => {
                    instr.push(asm::Instruction::AddImmediate {
                        dest: t_reg,
                        src: asm::GpOrSpRegister::GP(base_reg),
                        immediate: 1,
                        shift: 0,
                    });
                }
                ir::UnaryArithmeticOp::Negate => {
                    instr.push(asm::Instruction::NegateRegisterShifted {
                        dest: t_reg,
                        src1: base_reg,
                        shift: asm::Shift::LSL,
                        amount: 0,
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
