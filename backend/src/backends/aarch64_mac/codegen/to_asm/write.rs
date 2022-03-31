use crate::backends::aarch64_mac::{asm, codegen::Context};

pub fn write(addr_op: ir::Operand, value: ir::Operand, ctx: &Context) -> Vec<asm::Instruction> {
    let mut result = Vec::with_capacity(1);

    match addr_op {
        ir::Operand::Variable(var) => {
            write_var(var, value, ctx, &mut result);
        }
        ir::Operand::Constant(con) => {
            dbg!(&con);
            todo!()
        }
    };

    result
}

fn write_var(
    addr: ir::Variable,
    value: ir::Operand,
    ctx: &Context,
    instr: &mut Vec<asm::Instruction>,
) {
    let raw_reg = ctx.registers.get_reg(&addr).unwrap();
    let base_reg = match raw_reg {
        asm::Register::GeneralPurpose(asm::GPRegister::DWord(n)) => asm::GPRegister::DWord(n),
        other => {
            dbg!(&other);
            todo!()
        }
    };

    match value {
        ir::Operand::Constant(con) => match con {
            ir::Constant::I32(val) => {
                let val_register = asm::GPRegister::Word(9); // Register 9 should be a scratch register that can be used as seen fit
                instr.push(asm::Instruction::MovI64 {
                    dest: val_register.clone(),
                    value: val as i64,
                });

                instr.push(asm::Instruction::StoreRegisterUnscaled {
                    reg: val_register,
                    base: asm::GpOrSpRegister::GP(base_reg),
                    offset: asm::Imm9Signed::new(0),
                });
            }
            ir::Constant::U8(val) => {
                let val_register = asm::GPRegister::Word(9);
                instr.push(asm::Instruction::Movz {
                    dest: val_register.clone(),
                    immediate: val as u16,
                    shift: 0,
                });

                instr.push(asm::Instruction::StoreRegisterUnscaled {
                    reg: val_register,
                    base: asm::GpOrSpRegister::GP(base_reg),
                    offset: asm::Imm9Signed::new(0),
                });
            }
            other => {
                dbg!(&other);
                todo!()
            }
        },
        ir::Operand::Variable(var) => {
            let value_reg = ctx.registers.get_reg(&var).unwrap();

            match (value_reg, var.ty) {
                (asm::Register::GeneralPurpose(value), ir::Type::I32) => {
                    instr.push(asm::Instruction::StoreRegisterUnscaled {
                        reg: value,
                        base: asm::GpOrSpRegister::GP(base_reg),
                        offset: asm::Imm9Signed::new(0),
                    });
                }
                other_ty => {
                    dbg!(&other_ty);
                    todo!()
                }
            };
        }
    };
}
