use std::collections::HashMap;

use crate::{isas::sh4a, util};

use super::Target;

mod codegen;
mod stack;

pub struct Backend {}

impl Backend {
    pub fn new() -> Self {
        Self {}
    }

    fn avail_registers() -> [sh4a::Register; 15] {
        [
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(1)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(2)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(3)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(4)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(5)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(6)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(7)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(8)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(9)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(10)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(11)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(12)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(13)),
            sh4a::Register::GeneralPurpose(sh4a::GeneralPurposeRegister::new(14)),
            sh4a::Register::FloatingPoint(0),
        ]
    }

    fn codegen(
        &self,
        func: &ir::FunctionDefinition,
        register_map: HashMap<ir::Variable, sh4a::Register>,
    ) {
        let stack_allocation = util::stack::allocate_stack(
            func,
            &register_map,
            |space| {
                vec![
                    sh4a::Instruction::PushPR,
                    sh4a::Instruction::AddImmediate {
                        reg: sh4a::GeneralPurposeRegister::stack_reg(),
                        immediate: -(space as i8),
                    },
                ]
            },
            |space| {
                vec![
                    sh4a::Instruction::AddImmediate {
                        reg: sh4a::GeneralPurposeRegister::stack_reg(),
                        immediate: space as i8,
                    },
                    sh4a::Instruction::PopPR,
                ]
            },
            |register, offset| {
                let write_offset = offset + 4;

                match register {
                    sh4a::Register::GeneralPurpose(gp) => {
                        vec![
                            sh4a::Instruction::PushL {
                                reg: sh4a::GeneralPurposeRegister::new(0),
                            },
                            sh4a::Instruction::MovIR {
                                immediate: write_offset as i8,
                                dest: sh4a::GeneralPurposeRegister::new(0),
                            },
                            sh4a::Instruction::MovLRR0PR {
                                base: sh4a::GeneralPurposeRegister::stack_reg(),
                                src: gp.clone(),
                            },
                            sh4a::Instruction::PopL {
                                reg: sh4a::GeneralPurposeRegister::new(0),
                            },
                        ]
                    }
                    sh4a::Register::FloatingPoint(_) => {
                        todo!()
                    }
                    sh4a::Register::PR => {
                        todo!()
                    }
                }
            },
            |register, offset| {
                let read_offset = offset + 4;

                match register {
                    sh4a::Register::GeneralPurpose(gp) => {
                        vec![
                            sh4a::Instruction::PushL {
                                reg: sh4a::GeneralPurposeRegister::new(0),
                            },
                            sh4a::Instruction::MovIR {
                                immediate: read_offset as i8,
                                dest: sh4a::GeneralPurposeRegister::new(0),
                            },
                            sh4a::Instruction::MovLR0PRR {
                                base: sh4a::GeneralPurposeRegister::stack_reg(),
                                target: gp.clone(),
                            },
                            sh4a::Instruction::PopL {
                                reg: sh4a::GeneralPurposeRegister::new(0),
                            },
                        ]
                    }
                    sh4a::Register::FloatingPoint(fp) => {
                        todo!()
                    }
                    sh4a::Register::PR => {
                        todo!()
                    }
                }
            },
            |ty| match ty {
                ir::Type::I64 | ir::Type::U64 => (8, 8),
                ir::Type::I32 | ir::Type::U32 | ir::Type::Pointer(_) => (4, 4),
                other => {
                    dbg!(&other);
                    todo!()
                }
            },
            4,
            0,
        );
        dbg!(&stack_allocation);

        let ctx = codegen::Context {
            registers: register_map,
            var_offsets: stack_allocation.var_offsets,
            stack_allocs: stack_allocation.allocations,
            pre_ret_instr: stack_allocation.pre_return_instr,
        };

        let asm_blocks: Vec<_> = func
            .block
            .block_iter()
            .map(|b| codegen::block_to_asm(b, &ctx))
            .collect();
        dbg!(&asm_blocks);

        todo!("Codegen")
    }
}

impl Target for Backend {
    fn generate(&self, program: ir::Program) {
        let global_statements = program.global.get_statements();
        for stmnt in global_statements {
            dbg!(&stmnt);
            todo!()
        }

        let all_registers = Self::avail_registers();
        //let mut blocks = Vec::new();
        for (_, func) in program.functions.iter() {
            let registers = util::registers::allocate_registers(func, &all_registers);

            util::destructure::destructure_func(func);

            self.codegen(func, registers);
        }

        todo!()
    }
}
