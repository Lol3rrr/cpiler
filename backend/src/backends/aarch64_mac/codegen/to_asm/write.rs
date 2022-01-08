use crate::backends::aarch64_mac::{asm, codegen::Context, ArmRegister};

pub fn write(addr_op: ir::Operand, value: ir::Value, ctx: &Context) -> Vec<asm::Instruction> {
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
    value: ir::Value,
    ctx: &Context,
    instr: &mut Vec<asm::Instruction>,
) {
    dbg!(&addr, &value);

    let base_reg = match ctx.registers.get(&addr).unwrap() {
        ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
        ArmRegister::FloatingPoint(_) => todo!("Floating Point Register"),
    };
    dbg!(&base_reg);

    match value {
        ir::Value::Constant(con) => match con {
            ir::Constant::I32(val) => {
                let val_register = asm::GPRegister::Word(9); // Register 9 should be a scratch register that can be used as seen fit
                if val < 4096 && val >= 0 {
                    instr.push(asm::Instruction::Movz {
                        dest: val_register.clone(),
                        immediate: val as u16,
                        shift: 0,
                    });
                } else {
                    dbg!(&val);
                    todo!()
                }

                instr.push(asm::Instruction::StoreRegisterUnscaled {
                    reg: val_register,
                    base: asm::GpOrSpRegister::GP(base_reg),
                    offset: 0,
                });
            }
            other => {
                dbg!(&other);
                todo!()
            }
        },
        other => {
            dbg!(&other);
            todo!()
        }
    };
}
