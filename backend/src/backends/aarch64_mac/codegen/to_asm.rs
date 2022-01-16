use ir::{BasicBlock, Constant, Expression, Operand, Statement, Value};

use crate::backends::aarch64_mac::asm::{self, FloatImm8};

use super::Context;

mod binaryop;
mod function_call;
mod load;
mod unaryop;
mod write;

pub fn block_name(block: &BasicBlock) -> String {
    format!("block_{:x}", block.as_ptr() as usize)
}

pub fn block_to_asm(block: BasicBlock, ctx: &Context) -> asm::Block {
    let statements = block.get_statements();

    let name = block_name(&block);
    let mut instructions = Vec::new();

    for stmnt in statements {
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
                let src_reg = ctx.registers.get_reg(&src_var).unwrap();

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

                match (t_reg, con) {
                    (asm::Register::GeneralPurpose(t_reg), Constant::I32(val)) => {
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
                    (asm::Register::FloatingPoint(t_reg), Constant::F64(f_val)) => {
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
                };
            }
            Statement::Assignment {
                target,
                value: Value::Expression(exp),
            } => {
                let t_reg = ctx.registers.get_reg(&target).unwrap();

                match exp {
                    Expression::Cast { base, target } => {
                        match (t_reg, base) {
                            (asm::Register::GeneralPurpose(t_reg), Operand::Variable(base_var)) => {
                                let base_reg = ctx.registers.get_reg(&base_var).unwrap();
                                match base_reg {
                                    asm::Register::GeneralPurpose(gp) => {
                                        let base_reg = match (&t_reg, gp) {
                                            (
                                                asm::GPRegister::Word(_),
                                                asm::GPRegister::DWord(n),
                                            ) => asm::GPRegister::Word(n),
                                            (
                                                asm::GPRegister::DWord(_),
                                                asm::GPRegister::Word(n),
                                            ) => asm::GPRegister::DWord(n),
                                            (_, base) => base,
                                        };
                                        instructions.push(asm::Instruction::MovRegister {
                                            src: base_reg,
                                            dest: t_reg,
                                        });
                                    }
                                    asm::Register::FloatingPoint(fp) => {
                                        if target.signed() {
                                            instructions.push(asm::Instruction::FloatingPointToSignedIntegerMinusInf {
                                                src: fp,
                                                dest: t_reg,
                                            });
                                        } else {
                                            todo!("To unsigned integer");
                                        }
                                    }
                                };
                            }
                            (asm::Register::FloatingPoint(t_reg), Operand::Constant(con)) => {
                                dbg!(&t_reg, &con);

                                todo!()
                            }
                            (asm::Register::FloatingPoint(t_reg), Operand::Variable(base_var)) => {
                                let var_reg = ctx.registers.get_reg(&base_var).unwrap();
                                match var_reg {
                                    asm::Register::GeneralPurpose(base_reg) => {
                                        if base_var.ty.signed() {
                                            instructions.push(
                                                asm::Instruction::SignedIntegerToFloatingPoint {
                                                    src: base_reg,
                                                    dest: t_reg,
                                                },
                                            );
                                        } else {
                                            todo!()
                                        }
                                    }
                                    asm::Register::FloatingPoint(n) => {
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
                    ir::Expression::BinaryOp { op, left, right } => {
                        binaryop::to_asm(op, t_reg, left, right, ctx, &mut instructions);
                    }
                    ir::Expression::UnaryOp { op, base } => {
                        unaryop::to_asm(op, t_reg, base, ctx, &mut instructions);
                    }
                    ir::Expression::AdressOf {
                        base: ir::Operand::Variable(var),
                    } => {
                        dbg!(&var);

                        let offset = *ctx.var.get(&var.name).unwrap();
                        dbg!(&offset);

                        let t_reg = match t_reg {
                            asm::Register::GeneralPurpose(r) => r,
                            other => {
                                dbg!(&other);
                                panic!(
                                    "Addresses should never be stored in a Floating Point Register"
                                )
                            }
                        };

                        if (0..4096).contains(&offset) {
                            let addr_instr = asm::Instruction::AddImmediate {
                                dest: t_reg,
                                src: asm::GpOrSpRegister::SP,
                                immediate: offset as u16,
                                shift: 0,
                            };

                            instructions.push(addr_instr);
                        } else {
                            panic!()
                        }
                    }
                    ir::Expression::StackAlloc { .. } => {
                        let alloc_offset = *ctx.stack_allocs.get(&target).unwrap();

                        let t_reg = match t_reg {
                            asm::Register::GeneralPurpose(r) => r,
                            other => {
                                dbg!(&other);
                                panic!(
                                    "Addresses should never be stored in a Floating Point Register"
                                )
                            }
                        };

                        if (0..4096).contains(&alloc_offset) {
                            instructions.push(asm::Instruction::AddImmediate {
                                dest: t_reg,
                                src: asm::GpOrSpRegister::SP,
                                immediate: alloc_offset as u16,
                                shift: 0,
                            });
                        } else {
                            panic!()
                        }
                    }
                    ir::Expression::ReadMemory { address, read_ty } => {
                        dbg!(&address, &read_ty);

                        let base_reg = match address {
                            ir::Operand::Variable(base_var) => {
                                match ctx.registers.get_reg(&base_var).unwrap() {
                                    asm::Register::GeneralPurpose(asm::GPRegister::DWord(n)) => {
                                        asm::GPRegister::DWord(n)
                                    }
                                    other => {
                                        dbg!(&other);
                                        todo!()
                                    }
                                }
                            }
                            ir::Operand::Constant(base_con) => {
                                dbg!(&base_con);

                                todo!()
                            }
                        };

                        load::load(
                            t_reg,
                            read_ty,
                            asm::GpOrSpRegister::GP(base_reg),
                            0,
                            &mut instructions,
                        );
                    }
                    ir::Expression::FunctionCall {
                        name,
                        arguments,
                        return_ty,
                    } => {
                        function_call::to_asm(
                            name,
                            arguments,
                            return_ty,
                            Some(t_reg),
                            ctx,
                            &mut instructions,
                        );
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                };
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
                            asm::FPRegister::SinglePrecision(_) => {
                                asm::FPRegister::SinglePrecision(0)
                            }
                            asm::FPRegister::DoublePrecision(_) => {
                                asm::FPRegister::DoublePrecision(0)
                            }
                        };

                        instructions.push(asm::Instruction::FMovRegister { src, dest });
                    }
                };

                instructions.extend(ctx.pre_ret_instr.clone());
                instructions.push(asm::Instruction::Return);
            }
            ir::Statement::WriteMemory { target, value } => {
                let write_instr = write::write(target, value, ctx);
                instructions.extend(write_instr);
            }
            other => {
                dbg!(&other);
                todo!()
            }
        };
    }

    asm::Block { name, instructions }
}
