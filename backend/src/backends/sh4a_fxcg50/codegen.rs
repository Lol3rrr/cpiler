use std::collections::HashMap;

use crate::isas::sh4a;

mod constants;
mod expression;
mod inline_asm;

pub struct Context {
    pub registers: HashMap<ir::Variable, sh4a::Register>,
    pub var_offsets: HashMap<String, isize>,
    pub stack_allocs: HashMap<ir::Variable, isize>,
    pub pre_ret_instr: Vec<sh4a::Instruction>,
}

pub fn block_name(block: &ir::BasicBlock) -> String {
    format!("block_0x{:x}", block.as_ptr() as usize)
}

pub fn block_to_asm(block: ir::BasicBlock, ctx: &Context) -> sh4a::Block {
    let statements = block.get_statements();

    let name = block_name(&block);

    let mut instructions = Vec::new();

    for stmnt in statements {
        match stmnt {
            ir::Statement::Assignment {
                target,
                value: ir::Value::Variable(src_var),
            } => {
                let target_reg = ctx.registers.get(&target).unwrap().clone();
                let src_reg = ctx.registers.get(&src_var).unwrap().clone();

                match (target_reg, src_reg) {
                    (
                        sh4a::Register::GeneralPurpose(target),
                        sh4a::Register::GeneralPurpose(src),
                    ) => {
                        instructions.push(sh4a::Instruction::MovRR { dest: target, src });
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                };
            }
            ir::Statement::Assignment {
                target,
                value: ir::Value::Constant(con),
            } => {
                let target_reg = ctx.registers.get(&target).unwrap().clone();

                match (target_reg, con) {
                    (sh4a::Register::GeneralPurpose(gp), ir::Constant::I32(val)) => {
                        if (-128..127).contains(&val) {
                            instructions.push(sh4a::Instruction::MovIR {
                                dest: gp,
                                immediate: val as i8,
                            });

                            continue;
                        }

                        todo!("Constant I32")
                    }
                    (reg, con) => {
                        dbg!(&reg, &con);

                        todo!()
                    }
                };
            }
            ir::Statement::Assignment {
                target,
                value: ir::Value::Expression(exp),
            } => {
                let target_reg = ctx.registers.get(&target).unwrap().clone();

                let expression_instr = expression::to_asm(target_reg, exp, ctx);

                instructions.extend(expression_instr);
            }
            ir::Statement::SaveVariable { var } => {
                let var_reg = ctx.registers.get(&var).unwrap().clone();

                let stack_offset = ctx.var_offsets.get(&var.name).unwrap();

                // Save R0 as we use this for address calcuations
                instructions.push(sh4a::Instruction::PushL {
                    reg: sh4a::GeneralPurposeRegister::new(0),
                });

                let store_offset_instr =
                    constants::store_isize(sh4a::GeneralPurposeRegister::new(0), stack_offset + 4);

                instructions.extend(store_offset_instr);

                match var_reg {
                    sh4a::Register::GeneralPurpose(target) => {
                        instructions.push(sh4a::Instruction::MovLR0PRR {
                            base: sh4a::GeneralPurposeRegister::stack_reg(),
                            target,
                        });
                    }
                    sh4a::Register::FloatingPoint(_) => {
                        todo!("Save Float")
                    }
                    sh4a::Register::PR => {
                        todo!("Save PR")
                    }
                };

                // Restore R0
                instructions.push(sh4a::Instruction::PopL {
                    reg: sh4a::GeneralPurposeRegister::new(0),
                });
            }
            ir::Statement::Jump(target) => {
                let target_name = block_name(&target);

                instructions.push(sh4a::Instruction::JumpLabel { label: target_name });
            }
            ir::Statement::JumpTrue(var, target) => {
                let target_name = block_name(&target);

                let var_register = match ctx.registers.get(&var).unwrap().clone() {
                    sh4a::Register::GeneralPurpose(gp) => gp,
                    other => {
                        dbg!(&other);
                        panic!("Expected a GeneralPurpose Register for this")
                    }
                };

                instructions.push(sh4a::Instruction::CmpPl { src: var_register });
                instructions.push(sh4a::Instruction::BranchTrueLabel { label: target_name });
            }
            ir::Statement::Return(ret_var) => {
                if let Some(ret_var) = ret_var {
                    let ret_var_reg = ctx.registers.get(&ret_var).unwrap().clone();

                    match ret_var_reg {
                        sh4a::Register::GeneralPurpose(gp) => {
                            instructions.push(sh4a::Instruction::MovRR {
                                src: gp,
                                dest: sh4a::GeneralPurposeRegister::new(0),
                            });
                        }
                        other => {
                            dbg!(&other);
                            todo!()
                        }
                    };
                }

                instructions.push(sh4a::Instruction::Return);
            }
            ir::Statement::InlineAsm {
                template,
                inputs,
                output,
            } => {
                let input_regs: Vec<_> = inputs
                    .iter()
                    .map(|var| ctx.registers.get(var).unwrap().clone())
                    .collect();
                let output_reg = output
                    .as_ref()
                    .map(|var| ctx.registers.get(var).unwrap().clone());

                let asm_ctx = inline_asm::Context {
                    inputs: input_regs,
                    output: output_reg,
                };

                instructions.extend(inline_asm::convert(template, asm_ctx));
            }
            other => {
                dbg!(&other);
                todo!()
            }
        };
    }

    sh4a::Block::new(name, instructions)
}
