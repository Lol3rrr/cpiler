use ir::{Expression, Operand};

use crate::backends::aarch64_mac::{asm, codegen::load};

use super::{binaryop, function_call, unaryop, util, Context};

pub fn convert_assigned(
    exp: ir::Expression,
    target_var: ir::Variable,
    t_reg: asm::Register,
    ctx: &Context,
) -> Vec<asm::Instruction> {
    let mut instructions = Vec::new();

    match exp {
        Expression::Cast { base, target } => {
            match (t_reg, base) {
                (asm::Register::GeneralPurpose(t_reg), Operand::Variable(base_var)) => {
                    let base_reg = ctx.registers.get_reg(&base_var).unwrap();
                    match base_reg {
                        asm::Register::GeneralPurpose(gp) => {
                            let base_reg = match (&t_reg, gp) {
                                (asm::GPRegister::Word(_), asm::GPRegister::DWord(n)) => {
                                    asm::GPRegister::Word(n)
                                }
                                (asm::GPRegister::DWord(_), asm::GPRegister::Word(n)) => {
                                    asm::GPRegister::DWord(n)
                                }
                                (_, base) => base,
                            };
                            instructions.push(asm::Instruction::MovRegister {
                                src: base_reg,
                                dest: t_reg,
                            });
                        }
                        asm::Register::FloatingPoint(fp) => {
                            if target.signed() {
                                instructions.push(
                                    asm::Instruction::FloatingPointToSignedIntegerMinusInf {
                                        src: fp,
                                        dest: t_reg,
                                    },
                                );
                            } else {
                                todo!("To unsigned integer");
                            }
                        }
                    };
                }
                (asm::Register::GeneralPurpose(t_reg), Operand::Constant(con)) => {
                    let store_instr =
                        util::constant_to_asm(&con, asm::Register::GeneralPurpose(t_reg));
                    instructions.extend(store_instr);
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
                                instructions.push(asm::Instruction::SignedIntegerToFloatingPoint {
                                    src: base_reg,
                                    dest: t_reg,
                                });
                            } else {
                                todo!()
                            }
                        }
                        asm::Register::FloatingPoint(_n) => {
                            todo!()
                        }
                    };
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

            let offset = *ctx.var.get(var.name()).unwrap();
            dbg!(&offset);

            let t_reg = match t_reg {
                asm::Register::GeneralPurpose(r) => r,
                other => {
                    dbg!(&other);
                    panic!("Addresses should never be stored in a Floating Point Register")
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
            let alloc_offset = *ctx.stack_allocs.get(&target_var).unwrap();

            let t_reg = match t_reg {
                asm::Register::GeneralPurpose(r) => r,
                other => {
                    dbg!(&other);
                    panic!("Addresses should never be stored in a Floating Point Register")
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
        ir::Expression::ReadGlobalVariable { name } => {
            let addr_register = asm::GPRegister::DWord(9);

            instructions.push(asm::Instruction::Literal(format!(
                "adrp {}, {}@PAGE",
                addr_register, name,
            )));
            instructions.push(asm::Instruction::Literal(format!(
                "add {}, {}, {}@PAGEOFF",
                addr_register, addr_register, name
            )));

            match (t_reg, target_var.ty) {
                (asm::Register::GeneralPurpose(target), ir::Type::Pointer(_))
                | (asm::Register::GeneralPurpose(target), ir::Type::U32)
                | (asm::Register::GeneralPurpose(target), ir::Type::I32) => {
                    instructions.push(asm::Instruction::LoadRegisterUnscaled {
                        reg: target,
                        base: asm::GpOrSpRegister::GP(addr_register),
                        offset: asm::Imm9Signed::new(0).unwrap(),
                    });
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            };
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

    instructions
}
