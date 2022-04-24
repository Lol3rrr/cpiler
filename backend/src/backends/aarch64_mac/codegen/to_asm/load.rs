use crate::backends::aarch64_mac::{
    self,
    asm::{self},
    codegen::Context,
    ArmRegister,
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
                    match offset {
                        offset if asm::Imm9Signed::fits(offset) => {
                            let load_instr = asm::Instruction::LoadRegisterUnscaled {
                                reg: target_reg,
                                base,
                                offset: asm::Imm9Signed::new(offset).unwrap(),
                            };

                            instr.push(load_instr);
                        }
                        offset => {
                            let reg_number = match aarch64_mac::Backend::offset_register() {
                                ArmRegister::GeneralPurpose(n) => n,
                                ArmRegister::FloatingPoint(_) => {
                                    unreachable!("Got Floating-Point Register as Offset-Register");
                                }
                            };
                            let offset_reg = asm::GPRegister::DWord(reg_number);
                            let store_offset = asm::Instruction::MovI64 {
                                dest: offset_reg.clone(),
                                value: offset as i64,
                            };

                            let load_instr = asm::Instruction::LoadRegisterRegisterOffset {
                                reg: target_reg,
                                base,
                                offset: offset_reg,
                            };

                            instr.push(store_offset);
                            instr.push(load_instr);
                        }
                    };
                }
                ir::Type::I32 => {
                    let correct_target_reg = match target_reg {
                        asm::GPRegister::DWord(n) => asm::GPRegister::DWord(n),
                        asm::GPRegister::Word(n) => asm::GPRegister::DWord(n),
                    };

                    match offset {
                        offset if asm::Imm9Signed::fits(offset) => {
                            let load_instr = asm::Instruction::LoadSignedWordUnscaled {
                                reg: correct_target_reg,
                                base,
                                offset: asm::Imm9Signed::new(offset).unwrap(),
                            };

                            instr.push(load_instr);
                        }
                        offset => {
                            let reg_number = match aarch64_mac::Backend::offset_register() {
                                ArmRegister::GeneralPurpose(n) => n,
                                ArmRegister::FloatingPoint(_) => {
                                    unreachable!("Got Floating-Point Register as Offset-Register");
                                }
                            };
                            let offset_reg = asm::GPRegister::DWord(reg_number);
                            let store_offset = asm::Instruction::MovI64 {
                                dest: offset_reg.clone(),
                                value: offset as i64,
                            };

                            let load_instr = asm::Instruction::LoadSignedWordRegisterOffset {
                                reg: correct_target_reg,
                                base,
                                offset_reg,
                            };

                            instr.push(store_offset);
                            instr.push(load_instr);
                        }
                    };
                }
                ir::Type::I16 => {
                    instr.push(asm::Instruction::LoadSignedHalfWordUnscaled {
                        reg: target_reg,
                        base,
                        offset: asm::Imm9Signed::new(offset).unwrap(),
                    });
                }
                ir::Type::U16 => {
                    instr.push(asm::Instruction::LoadHalfWordUnscaled {
                        reg: target_reg,
                        base,
                        offset: asm::Imm9Signed::new(offset).unwrap(),
                    });
                }
                ir::Type::I8 => {
                    instr.push(asm::Instruction::LoadSignedByteUnscaled {
                        reg: target_reg,
                        base,
                        offset: asm::Imm9Signed::new(offset).unwrap(),
                    });
                }
                ir::Type::U8 => {
                    instr.push(asm::Instruction::LoadByteUnscaled {
                        reg: target_reg,
                        base,
                        offset: asm::Imm9Signed::new(offset).unwrap(),
                    });
                }
                _ => {}
            };
        }
        ir::Type::Void => {
            // TODO
            // We cant load a Void Type
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
            panic!("Unknown Offset for Variable: {:?}", var)
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
