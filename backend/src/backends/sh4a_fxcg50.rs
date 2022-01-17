//! # ABI:
//! ## Function-Call-Arguments:
//! TODO

// Instructions: http://shared-ptr.com/sh_insns.html
// General SH4: https://www.st.com/resource/en/user_manual/cd00147165-sh-4-32-bit-cpu-core-architecture-stmicroelectronics.pdf

use std::collections::HashMap;

use crate::{isas::sh4a, util};

use super::Target;

mod codegen;

pub struct Backend {}

impl Backend {
    pub fn new() -> Self {
        Self {}
    }

    fn avail_registers() -> [sh4a::Register; 11] {
        [
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
    ) -> Vec<sh4a::Block> {
        let stack_allocation = util::stack::allocate_stack(
            func,
            &register_map,
            |space| {
                let mut base = vec![sh4a::Instruction::PushPR];

                let mut space_left = space;
                while space_left > i8::MAX as usize {
                    base.push(sh4a::Instruction::AddImmediate {
                        reg: sh4a::GeneralPurposeRegister::stack_reg(),
                        immediate: -i8::MAX,
                    });
                    space_left -= i8::MAX as usize;
                }

                let space_left: i8 = space_left.try_into().unwrap();
                base.push(sh4a::Instruction::AddImmediate {
                    reg: sh4a::GeneralPurposeRegister::stack_reg(),
                    immediate: -space_left,
                });

                base
            },
            |space| {
                let mut base_alloc = Vec::new();
                let mut space_left = space;

                while space_left > i8::MAX as usize {
                    base_alloc.push(sh4a::Instruction::AddImmediate {
                        reg: sh4a::GeneralPurposeRegister::stack_reg(),
                        immediate: i8::MAX,
                    });
                    space_left -= i8::MAX as usize;
                }

                base_alloc.push(sh4a::Instruction::AddImmediate {
                    reg: sh4a::GeneralPurposeRegister::stack_reg(),
                    immediate: space_left.try_into().unwrap(),
                });

                base_alloc.push(sh4a::Instruction::PopPR);

                base_alloc
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

        let ctx = codegen::Context {
            registers: register_map,
            var_offsets: stack_allocation.var_offsets,
            stack_allocs: stack_allocation.allocations,
            pre_ret_instr: stack_allocation.pre_return_instr,
        };

        func.block
            .block_iter()
            .map(|b| codegen::block_to_asm(b, &ctx))
            .collect()
    }
}

impl Target for Backend {
    fn generate(&self, program: ir::Program) {
        let global_statements = program.global.get_statements();
        for stmnt in global_statements {
            dbg!(&stmnt);
        }

        let all_registers = Self::avail_registers();
        let mut blocks = Vec::new();
        for (_, func) in program.functions.iter() {
            let registers = util::registers::allocate_registers(func, &all_registers);

            util::destructure::destructure_func(func);

            let tmp = self.codegen(func, registers);
            blocks.extend(tmp);
        }

        let main_func = program.functions.get("main").unwrap();
        let main_first_block = &main_func.block;
        let main_block_name = codegen::block_name(main_first_block);

        let asm_code = sh4a::assembler::assemble(main_block_name, blocks);

        let mut g3a_builder = g3a::FileBuilder::new(
            "testing".to_string(),
            g3a::NaiveDateTime::new(
                g3a::NaiveDate::from_ymd(2022, 1, 15),
                g3a::NaiveTime::from_hms(0, 20, 0),
            ),
        );
        g3a_builder
            .code(asm_code)
            .internal_name("@TEST".to_string())
            .short_name("test".to_string());

        let g3a_file = g3a_builder.finish();

        let g3a_data = g3a_file.serialize("testing.g3a");

        std::fs::write("./testing.g3a", g3a_data).unwrap();
    }
}
