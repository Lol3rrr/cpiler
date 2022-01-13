use std::collections::HashMap;

use crate::isas::sh4a;

pub struct Context {
    pub registers: HashMap<ir::Variable, sh4a::Register>,
    pub var_offsets: HashMap<String, isize>,
    pub stack_allocs: HashMap<ir::Variable, isize>,
    pub pre_ret_instr: Vec<sh4a::Instruction>,
}

fn block_name(block: &ir::BasicBlock) -> String {
    format!("{:x}", block.as_ptr() as usize)
}

pub fn block_to_asm(block: ir::BasicBlock, ctx: &Context) -> sh4a::Block {
    let statements = block.get_statements();

    let name = block_name(&block);

    let mut instructions = Vec::new();

    for stmnt in statements {
        match stmnt {
            ir::Statement::Assignment {
                target,
                value: ir::Value::Constant(con),
            } => {
                let target_reg = ctx.registers.get(&target).unwrap().clone();

                match (target_reg, con) {
                    (sh4a::Register::GeneralPurpose(gp), ir::Constant::I32(val)) => {
                        dbg!(&gp, &val);

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
            ir::Statement::SaveVariable { var } => {
                dbg!(&var);
                let var_reg = ctx.registers.get(&var).unwrap().clone();

                todo!()
            }
            other => {
                dbg!(&other);
                todo!()
            }
        };
    }

    sh4a::Block::new(name, instructions)
}
