use crate::backends::aarch64_mac::{
    asm::{self},
    codegen::Context,
};

pub fn load(
    target_reg: asm::Register,
    read_ty: ir::Type,
    base: asm::GpOrSpRegister,
    offset: i16,
    instr: &mut Vec<asm::Instruction>,
) {
    match &read_ty {
        ir::Type::Pointer(_)
        | ir::Type::I64
        | ir::Type::U64
        | ir::Type::I32
        | ir::Type::U32
        | ir::Type::I16
        | ir::Type::U16
        | ir::Type::I8
        | ir::Type::U8 => {
            let target_reg = match target_reg {
                asm::Register::GeneralPurpose(r) => r,
                other => {
                    dbg!(&other);
                    panic!("")
                }
            };

            match &read_ty {
                ir::Type::I64 | ir::Type::U64 | ir::Type::Pointer(_) | ir::Type::U32 => {
                    let load_instr = asm::Instruction::LoadRegisterUnscaled {
                        reg: target_reg,
                        base,
                        offset: asm::Imm9Signed::new(offset),
                    };

                    instr.push(load_instr);
                }
                ir::Type::I32 => {
                    let correct_target_reg = match target_reg {
                        asm::GPRegister::DWord(n) => asm::GPRegister::DWord(n),
                        asm::GPRegister::Word(n) => asm::GPRegister::DWord(n),
                    };

                    let load_instr = match offset {
                        offset if asm::Imm9Signed::fits(offset) => {
                            asm::Instruction::LoadSignedWordUnscaled {
                                reg: correct_target_reg,
                                base,
                                offset: asm::Imm9Signed::new(offset),
                            }
                        }
                        offset => {
                            dbg!(offset);
                            todo!()
                        }
                    };

                    instr.push(load_instr);
                }
                ir::Type::I16 => {
                    instr.push(asm::Instruction::LoadSignedHalfWordUnscaled {
                        reg: target_reg,
                        base,
                        offset: asm::Imm9Signed::new(offset),
                    });
                }
                ir::Type::U16 => {
                    instr.push(asm::Instruction::LoadHalfWordUnscaled {
                        reg: target_reg,
                        base,
                        offset: asm::Imm9Signed::new(offset),
                    });
                }
                ir::Type::I8 => {
                    instr.push(asm::Instruction::LoadSignedByteUnscaled {
                        reg: target_reg,
                        base,
                        offset: asm::Imm9Signed::new(offset),
                    });
                }
                ir::Type::U8 => {
                    instr.push(asm::Instruction::LoadByteUnscaled {
                        reg: target_reg,
                        base,
                        offset: asm::Imm9Signed::new(offset),
                    });
                }
                _ => {}
            };
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
    let target_reg = match ctx.registers.get_reg(&var).unwrap() {
        asm::Register::GeneralPurpose(gp) => gp,
        other => {
            dbg!(&other);
            todo!()
        }
    };

    let base = asm::GpOrSpRegister::SP;
    let offset = match ctx.var.get(&var.name) {
        Some(o) => (*o).try_into().unwrap(),
        None => {
            dbg!(&var);
            panic!()
        }
    };

    load(
        asm::Register::GeneralPurpose(target_reg),
        read_ty,
        base,
        offset,
        instr,
    );
}
