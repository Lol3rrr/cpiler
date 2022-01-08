use ir::{BasicBlock, Constant, Expression, Operand, Statement, Value};

use crate::backends::aarch64_mac::{asm, ArmRegister};

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
                let src_reg = ctx.registers.get(&var).unwrap();
                let s_reg = match src_reg {
                    ArmRegister::GeneralPurpose(n) => match &var.ty {
                        ir::Type::I64 | ir::Type::U64 | ir::Type::Pointer(_) => {
                            asm::GPRegister::DWord(*n)
                        }
                        _ => asm::GPRegister::Word(*n),
                    },
                    ArmRegister::FloatingPoint(_) => todo!(),
                };

                let var_offset = *ctx.var.get(&var.name).unwrap();

                match var.ty {
                    ir::Type::I64 | ir::Type::U64 | ir::Type::Pointer(_) => {
                        instructions.push(asm::Instruction::StoreRegisterUnscaled {
                            reg: s_reg,
                            base: asm::GpOrSpRegister::SP,
                            offset: var_offset as i16,
                        });
                    }
                    ir::Type::I32 | ir::Type::U32 => {
                        instructions.push(asm::Instruction::StoreRegisterUnscaled {
                            reg: s_reg,
                            base: asm::GpOrSpRegister::SP,
                            offset: var_offset as i16,
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
                value: Value::Unknown,
            } => {
                let target_ty = target.ty.clone();
                load::load_var(target, target_ty, ctx, &mut instructions);
            }
            Statement::Assignment {
                target,
                value: Value::Variable(src_var),
            } => {
                let target_reg = ctx.registers.get(&target).unwrap();
                let src_reg = match ctx.registers.get(&src_var) {
                    Some(r) => r,
                    None => {
                        dbg!(&src_var, &ctx.registers);
                        dbg!(&block);

                        todo!()
                    }
                };

                let t_reg = match target_reg {
                    ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                    ArmRegister::FloatingPoint(n) => panic!("Not yet supported"),
                };
                let s_reg = match src_reg {
                    ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                    ArmRegister::FloatingPoint(n) => panic!("Not yet supported"),
                };

                instructions.push(asm::Instruction::MovRegister {
                    src: s_reg,
                    dest: t_reg,
                });
            }
            Statement::Assignment {
                target,
                value: Value::Constant(con),
            } => {
                dbg!(&target, &con);

                let target_reg = ctx.registers.get(&target).unwrap();
                let t_reg = match target_reg {
                    ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                    ArmRegister::FloatingPoint(n) => panic!("Not yet supported"),
                };

                match con {
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
                    other => todo!(),
                };
            }
            Statement::Assignment {
                target,
                value: Value::Expression(exp),
            } => {
                let target_reg = match ctx.registers.get(&target) {
                    Some(r) => r,
                    None => {
                        dbg!(&target, &ctx.registers);

                        todo!()
                    }
                };
                let t_reg = match target_reg {
                    ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                    ArmRegister::FloatingPoint(n) => panic!("Not yet supported"),
                };

                match exp {
                    Expression::Cast { base, target } => {
                        match base {
                            Operand::Variable(base_var) => {
                                // TODO
                                // Properly handle this

                                let base_reg = ctx.registers.get(&base_var).unwrap();
                                let b_reg = match base_reg {
                                    ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                                    ArmRegister::FloatingPoint(n) => panic!("Not yet supported"),
                                };

                                instructions.push(asm::Instruction::MovRegister {
                                    src: b_reg,
                                    dest: t_reg,
                                });
                            }
                            Operand::Constant(con) => todo!("Cast Const"),
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

                        if offset >= 0 && offset < 4096 {
                            let addr_instr = asm::Instruction::AddImmediate {
                                dest: t_reg,
                                src: asm::GpOrSpRegister::SP,
                                immediate: offset as u16,
                                shift: 0,
                            };

                            instructions.push(addr_instr);
                        }
                    }
                    ir::Expression::StackAlloc { .. } => {
                        let alloc_offset = *ctx.stack_allocs.get(&target).unwrap();

                        if alloc_offset >= 0 && alloc_offset < 4096 {
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
                                match ctx.registers.get(&base_var).unwrap() {
                                    ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                                    ArmRegister::FloatingPoint(_) => {
                                        todo!("Floating Point Register")
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

                let cond_reg = ctx.registers.get(&condition).unwrap();

                let c_reg = match cond_reg {
                    ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                    ArmRegister::FloatingPoint(n) => panic!("Not yet supported"),
                };

                instructions.push(asm::Instruction::BranchNonZeroLabel {
                    reg: c_reg,
                    target: target_name,
                });
            }
            Statement::Return(Some(ret_var)) => {
                dbg!(&ret_var);

                let ret_var_reg = ctx.registers.get(&ret_var).unwrap();
                let ret_reg = match ret_var_reg {
                    ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                    ArmRegister::FloatingPoint(n) => panic!("Not yet supported"),
                };

                // Set the correct Return Value
                instructions.push(asm::Instruction::MovRegister {
                    src: ret_reg,
                    dest: asm::GPRegister::DWord(0),
                });

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

pub fn stack_space<ISI, IS>(allocations: ISI) -> usize
where
    ISI: IntoIterator<IntoIter = IS, Item = (usize, usize)>,
    IS: Iterator<Item = (usize, usize)>,
{
    let mut base = 16;

    for (align, size) in allocations.into_iter() {
        if base % align != 0 {
            base += align - (base % align);
        }

        base += size;
    }

    if base % 16 == 0 {
        base
    } else {
        base + (16 - (base % 16))
    }
}
