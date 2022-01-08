// https://developer.apple.com/documentation/xcode/writing-arm64-code-for-apple-platforms
// https://developer.arm.com/documentation/102374/0101/Registers-in-AArch64---general-purpose-registers

use std::{
    collections::{HashMap, HashSet},
    process::Command,
};

use ir::Variable;

use crate::util;

use super::Target;

mod asm;
mod codegen;

pub struct Backend {}

impl Backend {
    pub fn new() -> Self {
        Self {}
    }

    /// General Purpose Registers
    fn registers() -> [ArmRegister; 42] {
        [
            //ArmRegister::GeneralPurpose(8),
            /*
            ArmRegister::GeneralPurpose(9),
            ArmRegister::GeneralPurpose(10),
            ArmRegister::GeneralPurpose(11),
            ArmRegister::GeneralPurpose(12),
            ArmRegister::GeneralPurpose(13),
            ArmRegister::GeneralPurpose(14),
            ArmRegister::GeneralPurpose(15),
            */
            //ArmRegister::GeneralPurpose(16),
            //ArmRegister::GeneralPurpose(17),
            //ArmRegister::GeneralPurpose(18),
            ArmRegister::GeneralPurpose(19),
            ArmRegister::GeneralPurpose(20),
            ArmRegister::GeneralPurpose(21),
            ArmRegister::GeneralPurpose(22),
            ArmRegister::GeneralPurpose(23),
            ArmRegister::GeneralPurpose(24),
            ArmRegister::GeneralPurpose(25),
            ArmRegister::GeneralPurpose(26),
            ArmRegister::GeneralPurpose(27),
            ArmRegister::GeneralPurpose(28),
            //ArmRegister::GeneralPurpose(29),
            //ArmRegister::GeneralPurpose(30),
            ArmRegister::FloatingPoint(0),
            ArmRegister::FloatingPoint(1),
            ArmRegister::FloatingPoint(2),
            ArmRegister::FloatingPoint(3),
            ArmRegister::FloatingPoint(4),
            ArmRegister::FloatingPoint(5),
            ArmRegister::FloatingPoint(6),
            ArmRegister::FloatingPoint(7),
            ArmRegister::FloatingPoint(8),
            ArmRegister::FloatingPoint(9),
            ArmRegister::FloatingPoint(10),
            ArmRegister::FloatingPoint(11),
            ArmRegister::FloatingPoint(12),
            ArmRegister::FloatingPoint(13),
            ArmRegister::FloatingPoint(14),
            ArmRegister::FloatingPoint(15),
            ArmRegister::FloatingPoint(16),
            ArmRegister::FloatingPoint(17),
            ArmRegister::FloatingPoint(18),
            ArmRegister::FloatingPoint(19),
            ArmRegister::FloatingPoint(20),
            ArmRegister::FloatingPoint(21),
            ArmRegister::FloatingPoint(22),
            ArmRegister::FloatingPoint(23),
            ArmRegister::FloatingPoint(24),
            ArmRegister::FloatingPoint(25),
            ArmRegister::FloatingPoint(26),
            ArmRegister::FloatingPoint(27),
            ArmRegister::FloatingPoint(28),
            ArmRegister::FloatingPoint(29),
            ArmRegister::FloatingPoint(30),
            ArmRegister::FloatingPoint(31),
        ]
    }

    fn codegen(
        &self,
        func: &ir::FunctionDefinition,
        register_map: HashMap<Variable, ArmRegister>,
    ) -> Vec<asm::Block> {
        let arg_targets = codegen::arguments(func.arguments.iter().map(|(_, t)| t.clone()));
        for (arg, arg_register) in func.arguments.iter().zip(arg_targets.iter()) {
            dbg!(&arg, &arg_register);
            todo!()
        }

        let stack_allocation = codegen::stack::allocate_stack(&func, &register_map);

        let asm_ctx = codegen::Context {
            registers: register_map,
            var: stack_allocation.var_offsets,
            pre_ret_instr: stack_allocation.pre_return_instr.clone(),
            stack_allocs: stack_allocation.allocations,
        };

        let mut asm_blocks: Vec<_> = func
            .block
            .block_iter()
            .map(|b| codegen::block_to_asm(b, &asm_ctx))
            .collect();

        asm_blocks.insert(
            0,
            asm::Block {
                name: func.name.clone(),
                instructions: vec![asm::Instruction::JumpLabel {
                    target: codegen::block_name(&func.block),
                }],
            },
        );

        {
            let first_block = asm_blocks.first_mut().unwrap();

            let n_instr: Vec<_> = stack_allocation
                .setup_instr
                .into_iter()
                .chain(std::mem::take(&mut first_block.instructions))
                .collect();
            first_block.instructions = n_instr;
        }

        asm_blocks
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ArmRegister {
    GeneralPurpose(u8),
    FloatingPoint(u8),
}

impl util::registers::Register for ArmRegister {
    fn reg_type(&self) -> util::registers::RegisterType {
        match self {
            Self::GeneralPurpose(_) => util::registers::RegisterType::GeneralPurpose,
            Self::FloatingPoint(_) => util::registers::RegisterType::FloatingPoint,
        }
    }
}

impl Target for Backend {
    fn generate(&self, program: ir::Program) {
        let all_registers = Self::registers();
        let mut blocks = Vec::new();
        for (_, func) in program.functions.iter() {
            let registers = util::registers::allocate_registers(func, &all_registers);

            util::destructure::destructure_func(func);

            let func_blocks = self.codegen(func, registers);
            blocks.extend(func_blocks);
        }

        let mut asm_text = "
.global main
.align 2
"
        .to_string();
        for block in blocks.iter() {
            let block_text = block.to_text();
            asm_text.push_str(&block_text);
        }

        std::fs::write("./code.s", asm_text);

        {
            let output = Command::new("as")
                .args(["-o", "./code.o", "./code.s"])
                .output()
                .expect("Failed to assemble");

            let err_str = String::from_utf8(output.stderr).unwrap();
            println!("{}", err_str);

            assert!(output.status.success());
        }

        {
            let output = Command::new("ld")
                .args(["-macosx_version_min", "12.0.0"])
                .args(["-o", "code"])
                .args(["code.o"])
                .args([
                    "-L/Library/Developer/CommandLineTools/SDKs/MacOSX12.sdk/usr/lib",
                    "-lSystem",
                    "-e",
                    "main",
                    "-arch",
                    "arm64",
                ])
                .output()
                .expect("Failed to link");

            let err_str = String::from_utf8(output.stderr).unwrap();
            println!("{}", err_str);

            assert!(output.status.success());
        }

        std::fs::remove_file("./code.s");
        std::fs::remove_file("./code.o");

        todo!()
    }
}
