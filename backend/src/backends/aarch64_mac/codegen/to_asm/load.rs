use crate::backends::aarch64_mac::{asm, codegen::Context, ArmRegister};

pub fn load(
    target_reg: asm::GPRegister,
    read_ty: ir::Type,
    base: asm::GpOrSpRegister,
    offset: i16,
    instr: &mut Vec<asm::Instruction>,
) {
    match read_ty {
        ir::Type::I64 | ir::Type::U64 | ir::Type::Pointer(_) | ir::Type::U32 => {
            let load_instr = asm::Instruction::LoadRegisterUnscaled {
                reg: target_reg,
                base,
                offset,
            };

            instr.push(load_instr);
        }
        ir::Type::I32 => {
            let load_instr = asm::Instruction::LoadSignedWordUnscaled {
                reg: target_reg,
                base,
                offset,
            };

            instr.push(load_instr);
        }
        ir::Type::I16 => {
            instr.push(asm::Instruction::LoadSignedHalfWordUnscaled {
                reg: target_reg,
                base,
                offset,
            });
        }
        ir::Type::U16 => {
            instr.push(asm::Instruction::LoadHalfWordUnscaled {
                reg: target_reg,
                base,
                offset,
            });
        }
        ir::Type::I8 => {
            instr.push(asm::Instruction::LoadSignedByteUnscaled {
                reg: target_reg,
                base,
                offset,
            });
        }
        ir::Type::U8 => {
            instr.push(asm::Instruction::LoadByteUnscaled {
                reg: target_reg,
                base,
                offset,
            });
        }
        other => {
            dbg!(&other);

            todo!()
        }
    };
}

pub fn load_var(
    var: ir::Variable,
    read_ty: ir::Type,
    ctx: &Context,
    instr: &mut Vec<asm::Instruction>,
) {
    let target_reg = match ctx.registers.get(&var).unwrap() {
        ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
        ArmRegister::FloatingPoint(_) => panic!("Floating Point Register"),
    };

    let base = asm::GpOrSpRegister::SP;
    let offset = *ctx.var.get(&var.name).unwrap() as i16;

    load(target_reg, read_ty, base, offset, instr);
}
