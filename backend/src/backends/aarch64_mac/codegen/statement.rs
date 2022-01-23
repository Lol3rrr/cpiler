use ir::{Constant, Statement, Value};

use crate::backends::aarch64_mac::asm::{self, FloatImm8};

use super::{block_name, expression, function_call, load, write, Context};

pub fn to_asm(stmnt: ir::Statement, ctx: &Context) -> Vec<asm::Instruction> {
    let mut instructions = Vec::new();

    match stmnt {
        Statement::SaveVariable { var } => {
            let src_reg = ctx.registers.get_reg(&var).unwrap();
            let var_offset = *ctx.var.get(&var.name).unwrap();
            let offset = asm::Imm9Signed::new(var_offset as i16);

            match src_reg {
                asm::Register::GeneralPurpose(gp) => match &var.ty {
                    ir::Type::I64 | ir::Type::U64 | ir::Type::Pointer(_) => {}
                    ir::Type::I32 | ir::Type::U32 => {
                        instructions.push(asm::Instruction::StoreRegisterUnscaled {
                            reg: gp,
                            base: asm::GpOrSpRegister::SP,
                            offset,
                        });
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                },
                asm::Register::FloatingPoint(fp) => match var.ty {
                    ir::Type::Float => {
                        instructions.push(asm::Instruction::StoreFPUnscaled {
                            reg: fp,
                            base: asm::GpOrSpRegister::SP,
                            offset,
                        });
                    }
                    ir::Type::Double => {
                        instructions.push(asm::Instruction::StoreFPUnscaled {
                            reg: fp,
                            base: asm::GpOrSpRegister::SP,
                            offset,
                        });
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                },
            };
        }
        Statement::SaveGlobalVariable { var } => {
            match ctx.registers.get_reg(&var).unwrap() {
                asm::Register::GeneralPurpose(gp) => {
                    let addr_reg = asm::GPRegister::DWord(9);

                    instructions.push(asm::Instruction::Literal(format!(
                        "adrp {}, {}@PAGE",
                        addr_reg, var.name,
                    )));
                    instructions.push(asm::Instruction::Literal(format!(
                        "add {}, {}, {}@PAGEOFF",
                        addr_reg, addr_reg, var.name
                    )));

                    dbg!(&gp);
                    match var.ty {
                        ir::Type::Pointer(_)
                        | ir::Type::U64
                        | ir::Type::I64
                        | ir::Type::U32
                        | ir::Type::I32 => {
                            //instructions
                            //    .push(asm::Instruction::Literal(format!("str {}, [x9]", gp)));

                            instructions.push(asm::Instruction::StoreRegisterUnscaled {
                                reg: gp,
                                base: asm::GpOrSpRegister::GP(addr_reg),
                                offset: asm::Imm9Signed::new(0),
                            });
                        }
                        other => {
                            dbg!(&other);
                            todo!()
                        }
                    };
                }
                asm::Register::FloatingPoint(fp) => {
                    dbg!(&fp);

                    todo!("Save Floating Point global")
                }
            };
        }
        Statement::Assignment {
            target,
            value: Value::Unknown,
        } => {
            let target_ty = target.ty.clone();
            load::load_var(target, target_ty, ctx, &mut instructions);
        }
        Statement::Assignment {
            target,
            value: Value::Variable(src_var),
        } => {
            let target_reg = ctx.registers.get_reg(&target).unwrap();
            let src_reg = match ctx.registers.get_reg(&src_var) {
                Some(r) => r,
                None => {
                    dbg!(&src_var);
                    panic!("No Register found for Src")
                }
            };

            match (target_reg, src_reg) {
                (asm::Register::GeneralPurpose(t), asm::Register::GeneralPurpose(s)) => {
                    instructions.push(asm::Instruction::MovRegister { src: s, dest: t });
                }
                (asm::Register::FloatingPoint(t), asm::Register::FloatingPoint(s)) => {
                    instructions.push(asm::Instruction::FMovRegister { src: s, dest: t });
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            };
        }
        Statement::Assignment {
            target,
            value: Value::Constant(con),
        } => {
            let t_reg = ctx.registers.get_reg(&target).unwrap();

            match t_reg {
                asm::Register::GeneralPurpose(t_reg) => match con {
                    Constant::I32(val) => {
                        if val < (i16::MAX as i32) && val >= 0 {
                            instructions.push(asm::Instruction::Movz {
                                dest: t_reg,
                                shift: 0,
                                immediate: val as u16,
                            });
                        } else {
                            todo!()
                        }
                    }
                    Constant::I64(val) => {
                        instructions.push(asm::Instruction::MovI64 {
                            dest: t_reg,
                            value: val,
                        });
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                },
                asm::Register::FloatingPoint(t_reg) => match con {
                    Constant::F64(f_val) => {
                        let immediate = FloatImm8::new(f_val as f32);

                        instructions.push(asm::Instruction::FMovImmediate {
                            dest: t_reg,
                            imm: immediate,
                        });
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                },
            };
        }
        Statement::Assignment {
            target,
            value: Value::Expression(exp),
        } => {
            let t_reg = ctx.registers.get_reg(&target).unwrap();

            let exp_instr = expression::convert_assigned(exp, target, t_reg, ctx);
            instructions.extend(exp_instr);
        }
        Statement::Jump(target) => {
            let target_name = block_name(&target);

            instructions.push(asm::Instruction::JumpLabel {
                target: target_name,
            });
        }
        Statement::JumpTrue(condition, target) => {
            let target_name = block_name(&target);

            let cond_reg = ctx.registers.get_reg(&condition).unwrap();
            let c_reg = match cond_reg {
                asm::Register::GeneralPurpose(n) => n,
                asm::Register::FloatingPoint(n) => panic!("Not yet supported"),
            };

            instructions.push(asm::Instruction::BranchNonZeroLabel {
                reg: c_reg,
                target: target_name,
            });
        }
        Statement::Return(Some(ret_var)) => {
            let ret_var_reg = ctx.registers.get_reg(&ret_var).unwrap();
            match ret_var_reg {
                asm::Register::GeneralPurpose(gp) => {
                    let src = gp.clone();
                    let dest = match gp {
                        asm::GPRegister::DWord(_) => asm::GPRegister::DWord(0),
                        asm::GPRegister::Word(_) => asm::GPRegister::Word(0),
                    };

                    instructions.push(asm::Instruction::MovRegister { src, dest });
                }
                asm::Register::FloatingPoint(fp) => {
                    let src = fp.clone();
                    let dest = match fp {
                        asm::FPRegister::SinglePrecision(_) => asm::FPRegister::SinglePrecision(0),
                        asm::FPRegister::DoublePrecision(_) => asm::FPRegister::DoublePrecision(0),
                    };

                    instructions.push(asm::Instruction::FMovRegister { src, dest });
                }
            };

            instructions.extend(ctx.pre_ret_instr.clone());
            instructions.push(asm::Instruction::Return);
        }
        ir::Statement::Return(None) => {
            instructions.extend(ctx.pre_ret_instr.clone());
            instructions.push(asm::Instruction::Return);
        }
        ir::Statement::WriteMemory { target, value } => {
            let write_instr = write::write(target, value, ctx);
            instructions.extend(write_instr);
        }
        ir::Statement::Call { name, arguments } => {
            function_call::to_asm(
                name,
                arguments,
                ir::Type::Void,
                None,
                ctx,
                &mut instructions,
            );
        }
        other => {
            dbg!(&other);
            todo!()
        }
    };

    instructions
}
