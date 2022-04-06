use super::Context;
use crate::{backends::sh4a_fxcg50::codegen::constants, isas::sh4a};

fn oper_to_reg(
    operand: ir::Operand,
    other_used: Option<sh4a::Register>,
    ctx: &Context,
) -> (sh4a::Register, Vec<sh4a::Instruction>, bool) {
    match operand {
        ir::Operand::Variable(var) => (ctx.registers.get(&var).unwrap().clone(), Vec::new(), false),
        ir::Operand::Constant(con) => {
            let free_reg = match con {
                ir::Constant::F32(_) | ir::Constant::F64(_) => {
                    todo!("Floating Point Register");
                }
                _ => {
                    let r0 = sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(0));

                    match other_used {
                        Some(other) if other == r0 => {
                            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(1))
                        }
                        _ => r0,
                    }
                }
            };
            dbg!(&free_reg);

            let store_instr = match (con, free_reg.clone()) {
                (ir::Constant::I64(val), sh4a::Register::GeneralPurpose(target)) => {
                    constants::store_i64(target, val)
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            };

            (free_reg, store_instr, true)
        }
    }
}

pub fn to_asm(
    _target_var: &ir::Variable,
    target: sh4a::Register,
    expression: ir::Expression,
    ctx: &Context,
) -> Vec<sh4a::Instruction> {
    dbg!(&target, &expression);

    match expression {
        ir::Expression::BinaryOp { op, left, right } => {
            let mut result = Vec::new();

            let (left_reg, left_init, left_save) = oper_to_reg(left.clone(), None, ctx);
            if left_save {
                todo!()
            }
            result.extend(left_init);

            let (right_reg, right_init, right_save) =
                oper_to_reg(right, Some(left_reg.clone()), ctx);
            if right_save {
                match right_reg.clone() {
                    sh4a::Register::GeneralPurpose(gp) => {
                        result.push(sh4a::Instruction::PushL { reg: gp });
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                };
            }
            result.extend(right_init);

            let op_instr: Vec<_> = match op {
                ir::BinaryOp::Logic(log_op) => {
                    let left_ty = left.ty();
                    let signed = left_ty.signed();

                    match (target, left_reg.clone(), right_reg.clone(), log_op) {
                        (
                            sh4a::Register::GeneralPurpose(target),
                            sh4a::Register::GeneralPurpose(left),
                            sh4a::Register::GeneralPurpose(right),
                            ir::BinaryLogicOp::Greater,
                        ) if signed => vec![
                            sh4a::Instruction::CmpGt { left, right },
                            sh4a::Instruction::MovT { dest: target },
                        ],
                        other => {
                            dbg!(&other);
                            todo!()
                        }
                    }
                }
                ir::BinaryOp::Arith(arith_op) => {
                    let is_float = left.ty().is_float();

                    match (arith_op, left_reg.clone(), right_reg.clone(), target) {
                        (
                            ir::BinaryArithmeticOp::Sub,
                            sh4a::Register::GeneralPurpose(left),
                            sh4a::Register::GeneralPurpose(right),
                            sh4a::Register::GeneralPurpose(target),
                        ) if !is_float => {
                            vec![
                                sh4a::Instruction::MovRR {
                                    src: left,
                                    dest: target.clone(),
                                },
                                sh4a::Instruction::Sub {
                                    dest: target,
                                    src2: right,
                                },
                            ]
                        }
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

            result.extend(op_instr);

            if right_save {
                match right_reg {
                    sh4a::Register::GeneralPurpose(gp) => {
                        result.push(sh4a::Instruction::PopL { reg: gp });
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                };
            }
            if left_save {
                dbg!(&left_reg);
                todo!()
            }

            result
        }
        ir::Expression::Cast {
            base,
            target: target_ty,
        } => {
            let mut result = Vec::new();

            let base_ty = base.ty();

            let (base_reg, base_init, base_save) = oper_to_reg(base, None, ctx);
            if base_save {
                match base_reg.clone() {
                    sh4a::Register::GeneralPurpose(gp) => {
                        result.push(sh4a::Instruction::PushL { reg: gp })
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                };
            }
            result.extend(base_init);

            match (target, base_reg.clone(), target_ty, base_ty) {
                (
                    sh4a::Register::GeneralPurpose(target),
                    sh4a::Register::GeneralPurpose(base),
                    ir::Type::I64,
                    ir::Type::I32,
                )
                | (
                    sh4a::Register::GeneralPurpose(target),
                    sh4a::Register::GeneralPurpose(base),
                    ir::Type::I32,
                    ir::Type::I64,
                ) => {
                    result.push(sh4a::Instruction::MovRR {
                        dest: target,
                        src: base,
                    });
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            }

            if base_save {
                match base_reg {
                    sh4a::Register::GeneralPurpose(gp) => {
                        result.push(sh4a::Instruction::PopL { reg: gp })
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                };
            }

            result
        }
        ir::Expression::StackAlloc { size, alignment } => {
            dbg!(&target, &size, &alignment);

            todo!()
        }
        other => {
            dbg!(&other);

            todo!()
        }
    }
}
