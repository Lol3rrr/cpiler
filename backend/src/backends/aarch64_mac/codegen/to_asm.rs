use std::collections::{HashMap, HashSet};

use ir::{
    BasicBlock, BinaryArithmeticOp, BinaryLogicOp, BinaryOp, Constant, Expression, Operand,
    Statement, Value, Variable,
};

use crate::backends::aarch64_mac::{asm, ArmRegister};

pub fn block_name(block: &BasicBlock) -> String {
    format!("block_{:x}", block.as_ptr() as usize)
}

pub fn block_to_asm(
    block: BasicBlock,
    reg_map: &HashMap<Variable, ArmRegister>,
    pre_ret_instr: Vec<asm::Instruction>,
) -> asm::Block {
    let statements = block.get_statements();

    let name = block_name(&block);
    let mut instructions = Vec::new();

    for stmnt in statements {
        match stmnt {
            Statement::Assignment {
                target,
                value: Value::Variable(src_var),
            } => {
                let target_reg = reg_map.get(&target).unwrap();
                let src_reg = reg_map.get(&src_var).unwrap();

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

                let target_reg = reg_map.get(&target).unwrap();
                let t_reg = match target_reg {
                    ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                    ArmRegister::FloatingPoint(n) => panic!("Not yet supported"),
                };

                match con {
                    Constant::I32(val) => {
                        if val < (i16::MAX as i32) && val > 0 {
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
                let target_reg = reg_map.get(&target).unwrap();
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

                                let base_reg = reg_map.get(&base_var).unwrap();
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
                    Expression::BinaryOp { op, left, right } => {
                        match op {
                            BinaryOp::Logic(log_op) => {
                                match (&left, &right) {
                                    (Operand::Variable(var), Operand::Constant(con)) => {
                                        let var_reg = match reg_map.get(&var).unwrap() {
                                            ArmRegister::GeneralPurpose(n) => {
                                                asm::GPRegister::DWord(*n)
                                            }
                                            ArmRegister::FloatingPoint(n) => {
                                                panic!("Not yet supported")
                                            }
                                        };
                                        let immediate = match con {
                                            Constant::I64(val) => {
                                                if *val >= 0 && *val < 4096 {
                                                    *val as u16
                                                } else {
                                                    panic!()
                                                }
                                            }
                                            other => {
                                                dbg!(&other);
                                                todo!()
                                            }
                                        };

                                        instructions.push(asm::Instruction::CmpImmediate {
                                            reg: var_reg,
                                            immediate,
                                            shift: 0,
                                        });
                                    }
                                    other => {
                                        dbg!(&other);
                                        todo!()
                                    }
                                };

                                let condition = match log_op {
                                    BinaryLogicOp::Greater => {
                                        if left.ty().signed() {
                                            asm::Cond::Gt
                                        } else {
                                            todo!("Unsigned Greater than comparison")
                                        }
                                    }
                                    other => {
                                        dbg!(&other);
                                        todo!()
                                    }
                                };

                                instructions.push(asm::Instruction::CSet {
                                    target: t_reg,
                                    condition,
                                });
                            }
                            BinaryOp::Arith(arith_op) => {
                                match (arith_op, left, right) {
                                    (
                                        BinaryArithmeticOp::Sub,
                                        Operand::Variable(var),
                                        Operand::Constant(con),
                                    ) => {
                                        let var_reg = match reg_map.get(&var).unwrap() {
                                            ArmRegister::GeneralPurpose(n) => {
                                                asm::GPRegister::DWord(*n)
                                            }
                                            ArmRegister::FloatingPoint(n) => {
                                                panic!("Not yet supported")
                                            }
                                        };

                                        let immediate = match con {
                                            Constant::I64(val) => {
                                                if val >= 0 && val < 4096 {
                                                    val as u16
                                                } else {
                                                    panic!()
                                                }
                                            }
                                            other => {
                                                dbg!(&other);
                                                todo!()
                                            }
                                        };

                                        instructions.push(asm::Instruction::SubImmediate {
                                            dest: t_reg,
                                            src: var_reg,
                                            immediate,
                                            shift: 0,
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

                let cond_reg = reg_map.get(&condition).unwrap();

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

                let ret_var_reg = reg_map.get(&ret_var).unwrap();
                let ret_reg = match ret_var_reg {
                    ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                    ArmRegister::FloatingPoint(n) => panic!("Not yet supported"),
                };

                // Set the correct Return Value
                instructions.push(asm::Instruction::MovRegister {
                    src: ret_reg,
                    dest: asm::GPRegister::DWord(0),
                });

                instructions.extend(pre_ret_instr.clone());
                instructions.push(asm::Instruction::Return);
            }
            other => {
                dbg!(&other);
                todo!()
            }
        };
    }

    asm::Block { name, instructions }
}

pub fn stack_space(func: &ir::FunctionDefinition, used_register: &HashSet<ArmRegister>) -> usize {
    let mut base = 16;

    for reg in used_register.iter() {
        match reg {
            ArmRegister::GeneralPurpose(_) => {
                base += 8;
            }
            ArmRegister::FloatingPoint(_) => {
                todo!()
            }
        };
    }

    if base % 16 == 0 {
        base
    } else {
        base + (16 - (base % 16))
    }
}
