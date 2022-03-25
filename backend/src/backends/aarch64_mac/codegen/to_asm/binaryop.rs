use crate::backends::aarch64_mac::{
    asm,
    codegen::{util, Context},
};

fn operand(
    oper: ir::Operand,
    ctx: &Context,
    used: Option<asm::Register>,
) -> (asm::Register, Vec<asm::Instruction>) {
    match oper {
        ir::Operand::Variable(var) => {
            let reg = ctx.registers.get_reg(&var).unwrap();
            (reg, Vec::new())
        }
        ir::Operand::Constant(con) => {
            dbg!(&con);
            todo!()
        }
    }
}

pub fn to_asm(
    op: ir::BinaryOp,
    t_reg: asm::Register,
    left: ir::Operand,
    right: ir::Operand,
    ctx: &Context,
    instructions: &mut Vec<asm::Instruction>,
) {
    match op {
        ir::BinaryOp::Logic(log_op) => {
            let signed = left.ty().signed();
            let float = left.ty().is_float();
            let (condition, swap_sites) = match log_op {
                ir::BinaryLogicOp::Greater if !float && signed => (asm::Cond::Gt, false),
                ir::BinaryLogicOp::Greater if !float && !signed => (asm::Cond::Hi, false),
                ir::BinaryLogicOp::Greater if float => (asm::Cond::Gt, false),
                ir::BinaryLogicOp::GreaterEq if !float && signed => (asm::Cond::Ge, false),
                ir::BinaryLogicOp::GreaterEq if !float && !signed => (asm::Cond::Ls, true),
                ir::BinaryLogicOp::GreaterEq if float => (asm::Cond::Ge, false),
                ir::BinaryLogicOp::Less if !float && signed => (asm::Cond::Lt, false),
                ir::BinaryLogicOp::Less if !float && !signed => (asm::Cond::Hi, true),
                ir::BinaryLogicOp::Less if float => (asm::Cond::Mi, false),
                ir::BinaryLogicOp::LessEq if !float && signed => (asm::Cond::Le, false),
                ir::BinaryLogicOp::LessEq if !float && !signed => (asm::Cond::Ls, false),
                ir::BinaryLogicOp::LessEq if float => (asm::Cond::Ls, false),
                ir::BinaryLogicOp::Equal => (asm::Cond::Equal, false),
                ir::BinaryLogicOp::NotEqual => (asm::Cond::NotEqual, false),
                other => {
                    dbg!(&other, &signed, &float);
                    unreachable!("All Patterns should be covered");
                }
            };

            match (&left, &right) {
                (ir::Operand::Variable(var), ir::Operand::Constant(con)) => {
                    let (var_reg, cmp_reg) = match ctx.registers.get_reg(var).unwrap() {
                        asm::Register::GeneralPurpose(gp) => match &gp {
                            asm::GPRegister::DWord(_) => (gp, asm::GPRegister::DWord(9)),
                            asm::GPRegister::Word(_) => (gp, asm::GPRegister::Word(9)),
                        },
                        asm::Register::FloatingPoint(fp) => {
                            todo!()
                        }
                    };

                    let store_cond_instr =
                        util::constant_to_asm(con, asm::Register::GeneralPurpose(cmp_reg.clone()));

                    instructions.extend(store_cond_instr);

                    let (first, second) = if !swap_sites {
                        (var_reg, cmp_reg)
                    } else {
                        (cmp_reg, var_reg)
                    };

                    instructions.push(asm::Instruction::CmpShifted {
                        first,
                        second,
                        shift: asm::Shift::LSL,
                        amount: 0,
                    });
                }
                (ir::Operand::Variable(l_var), ir::Operand::Variable(r_var)) => {
                    let l_var_reg = ctx.registers.get_reg(&l_var).unwrap();
                    let r_var_reg = ctx.registers.get_reg(&r_var).unwrap();

                    match (l_var_reg, r_var_reg) {
                        (
                            asm::Register::GeneralPurpose(l_gp),
                            asm::Register::GeneralPurpose(r_gp),
                        ) => {
                            let l_gp = match &l_gp {
                                asm::GPRegister::DWord(_) => l_gp,
                                asm::GPRegister::Word(n) => asm::GPRegister::DWord(*n),
                            };
                            let r_gp = match &r_gp {
                                asm::GPRegister::DWord(_) => r_gp,
                                asm::GPRegister::Word(n) => asm::GPRegister::DWord(*n),
                            };

                            let (first, second) = if !swap_sites {
                                (l_gp, r_gp)
                            } else {
                                (r_gp, l_gp)
                            };

                            instructions.push(asm::Instruction::CmpShifted {
                                first,
                                second,
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

            match t_reg {
                asm::Register::GeneralPurpose(t_reg) => {
                    instructions.push(asm::Instruction::CSet {
                        target: t_reg,
                        condition,
                    });
                }
                asm::Register::FloatingPoint(n) => panic!("Condition in FP-Register"),
            };
        }
        ir::BinaryOp::Arith(arith_op) => {
            let (first_reg, second_reg) = match (left, right) {
                (ir::Operand::Variable(var), ir::Operand::Constant(con)) => {
                    let (var_reg, imm_reg) = match ctx.registers.get_reg(&var).unwrap() {
                        asm::Register::GeneralPurpose(gp) => match &gp {
                            asm::GPRegister::DWord(_) => (
                                asm::Register::GeneralPurpose(gp),
                                asm::Register::GeneralPurpose(asm::GPRegister::DWord(9)),
                            ),
                            asm::GPRegister::Word(_) => (
                                asm::Register::GeneralPurpose(gp),
                                asm::Register::GeneralPurpose(asm::GPRegister::Word(9)),
                            ),
                        },
                        asm::Register::FloatingPoint(fp) => {
                            // TODO
                            // Figure out which register to use for this Floating Point Value
                            match &fp {
                                asm::FPRegister::SinglePrecision(_) => (
                                    asm::Register::FloatingPoint(fp),
                                    asm::Register::FloatingPoint(asm::FPRegister::SinglePrecision(
                                        30,
                                    )),
                                ),
                                asm::FPRegister::DoublePrecision(_) => (
                                    asm::Register::FloatingPoint(fp),
                                    asm::Register::FloatingPoint(asm::FPRegister::DoublePrecision(
                                        30,
                                    )),
                                ),
                            }
                        }
                    };

                    let imm_store_instr = util::constant_to_asm(&con, imm_reg.clone());
                    instructions.extend(imm_store_instr);

                    (var_reg, imm_reg)
                }
                (ir::Operand::Variable(first_var), ir::Operand::Variable(second_var)) => {
                    let first_var_reg: asm::Register = ctx.registers.get_reg(&first_var).unwrap();
                    let second_var_reg: asm::Register = ctx.registers.get_reg(&second_var).unwrap();

                    match (&first_var_reg, &second_var_reg) {
                        (
                            asm::Register::GeneralPurpose(l_gp),
                            asm::Register::GeneralPurpose(r_gp),
                        ) => match (&l_gp, &r_gp) {
                            (asm::GPRegister::DWord(_), asm::GPRegister::DWord(_)) => {
                                (first_var_reg, second_var_reg)
                            }
                            (asm::GPRegister::DWord(_), asm::GPRegister::Word(n)) => (
                                first_var_reg,
                                asm::Register::GeneralPurpose(asm::GPRegister::DWord(*n)),
                            ),
                            (asm::GPRegister::Word(_), asm::GPRegister::Word(_)) => {
                                (first_var_reg, second_var_reg)
                            }
                            other => {
                                dbg!(&other);
                                todo!()
                            }
                        },
                        (
                            asm::Register::FloatingPoint(l_fp),
                            asm::Register::FloatingPoint(r_fp),
                        ) => match (&l_fp, &r_fp) {
                            (
                                asm::FPRegister::SinglePrecision(_),
                                asm::FPRegister::SinglePrecision(_),
                            ) => (first_var_reg, second_var_reg),
                            other => {
                                dbg!(&other);
                                todo!()
                            }
                        },
                        other => {
                            dbg!(&other);
                            todo!()
                        }
                    }
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            };

            match arith_op {
                ir::BinaryArithmeticOp::Add => {
                    match (t_reg, first_reg, second_reg) {
                        (
                            asm::Register::GeneralPurpose(t_reg),
                            asm::Register::GeneralPurpose(first_reg),
                            asm::Register::GeneralPurpose(second_reg),
                        ) => {
                            instructions.push(asm::Instruction::AddRegisterShifted {
                                dest: t_reg,
                                src1: first_reg,
                                src2: second_reg,
                                shift: asm::Shift::LSL,
                                amount: 0,
                            });
                        }
                        (
                            asm::Register::FloatingPoint(t_reg),
                            asm::Register::FloatingPoint(first_reg),
                            asm::Register::FloatingPoint(second_reg),
                        ) => {
                            instructions.push(asm::Instruction::FPAdd {
                                dest: t_reg,
                                src1: first_reg,
                                src2: second_reg,
                            });
                        }
                        other => {
                            dbg!(&other);
                            todo!()
                        }
                    };
                }
                ir::BinaryArithmeticOp::Sub => {
                    match (t_reg, first_reg, second_reg) {
                        (
                            asm::Register::GeneralPurpose(t_reg),
                            asm::Register::GeneralPurpose(first_reg),
                            asm::Register::GeneralPurpose(second_reg),
                        ) => {
                            instructions.push(asm::Instruction::SubRegisterShifted {
                                dest: t_reg,
                                src1: first_reg,
                                src2: second_reg,
                                shift: asm::Shift::LSL,
                                amount: 0,
                            });
                        }
                        (
                            asm::Register::FloatingPoint(t_reg),
                            asm::Register::FloatingPoint(first_reg),
                            asm::Register::FloatingPoint(second_reg),
                        ) => {
                            instructions.push(asm::Instruction::FPSub {
                                dest: t_reg,
                                src1: first_reg,
                                src2: second_reg,
                            });
                        }
                        other => {
                            dbg!(&other);
                            todo!()
                        }
                    };
                }
                ir::BinaryArithmeticOp::Multiply => {
                    match (t_reg, first_reg, second_reg) {
                        (
                            asm::Register::GeneralPurpose(t_reg),
                            asm::Register::GeneralPurpose(first_reg),
                            asm::Register::GeneralPurpose(second_reg),
                        ) => {
                            instructions.push(asm::Instruction::MultiplyRegister {
                                dest: t_reg,
                                src1: first_reg,
                                src2: second_reg,
                            });
                        }
                        (
                            asm::Register::FloatingPoint(t_reg),
                            asm::Register::FloatingPoint(first_reg),
                            asm::Register::FloatingPoint(second_reg),
                        ) => {
                            instructions.push(asm::Instruction::FPMul {
                                dest: t_reg,
                                src1: first_reg,
                                src2: second_reg,
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
        ir::BinaryOp::LogicCombinator(combinator) => {
            let (left_reg, left_setup) = operand(left, ctx, None);
            instructions.extend(left_setup);

            let (right_reg, right_setup) = operand(right, ctx, Some(left_reg.clone()));
            instructions.extend(right_setup);

            dbg!(&left_reg, &right_reg);

            let (t_reg, f_reg, s_reg) = match (t_reg, left_reg, right_reg) {
                (
                    asm::Register::GeneralPurpose(t),
                    asm::Register::GeneralPurpose(f),
                    asm::Register::GeneralPurpose(s),
                ) => (t, f, s),
                other => {
                    dbg!(&other);
                    todo!()
                }
            };

            match combinator {
                ir::BinaryLogicCombinator::Or => {
                    instructions.push(asm::Instruction::BitwiseOrRegisterShifted {
                        dest: t_reg,
                        src1: f_reg,
                        src2: s_reg,
                        shift: asm::Shift::LSL,
                        amount: 0,
                    });
                }
                ir::BinaryLogicCombinator::And => {
                    todo!("And-Combinator")
                }
            };
        }
        other => {
            dbg!(&other);
            todo!()
        }
    };
}
