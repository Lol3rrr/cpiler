// https://developer.apple.com/documentation/xcode/writing-arm64-code-for-apple-platforms
// https://developer.arm.com/documentation/102374/0101/Registers-in-AArch64---general-purpose-registers
// General: https://developer.arm.com/documentation/ddi0487/latest/
// Call ABI: https://developer.arm.com/documentation/ihi0055/b/

use std::{collections::HashMap, process::Command};

use ir::Variable;

use crate::{backends::aarch64_mac::codegen::ArgTarget, util};

use super::Target;

mod asm;
mod codegen;

pub struct Backend {}

impl Backend {
    pub fn new() -> Self {
        Self {}
    }

    /// General Purpose Registers
    fn registers() -> [ArmRegister; 34] {
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
            /*
             * I think these are mainly used in Function Calls and I therefor would rather save
             * them just to be save
            ArmRegister::FloatingPoint(0),
            ArmRegister::FloatingPoint(1),
            ArmRegister::FloatingPoint(2),
            ArmRegister::FloatingPoint(3),
            ArmRegister::FloatingPoint(4),
            ArmRegister::FloatingPoint(5),
            ArmRegister::FloatingPoint(6),
            ArmRegister::FloatingPoint(7),
            */
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
        let stack_allocation = util::stack::allocate_stack(
            func,
            &register_map,
            |space| {
                vec![asm::Instruction::StpPreIndex {
                    first: asm::GPRegister::DWord(29),
                    second: asm::GPRegister::DWord(30),
                    base: asm::GpOrSpRegister::SP,
                    offset: -(space as i16),
                }]
            },
            |space| {
                vec![asm::Instruction::LdpPostIndex {
                    first: asm::GPRegister::DWord(29),
                    second: asm::GPRegister::DWord(30),
                    base: asm::GpOrSpRegister::SP,
                    offset: space as i16,
                }]
            },
            |register, offset| match register {
                ArmRegister::GeneralPurpose(n) => vec![asm::Instruction::StoreRegisterUnscaled {
                    reg: asm::GPRegister::DWord(*n),
                    base: asm::GpOrSpRegister::SP,
                    offset: asm::Imm9Signed::new(offset),
                }],
                ArmRegister::FloatingPoint(n) => vec![asm::Instruction::StoreFPUnscaled {
                    reg: asm::FPRegister::DoublePrecision(*n),
                    base: asm::GpOrSpRegister::SP,
                    offset: asm::Imm9Signed::new(offset),
                }],
            },
            |register, offset| match register {
                ArmRegister::GeneralPurpose(n) => vec![asm::Instruction::LoadRegisterUnscaled {
                    reg: asm::GPRegister::DWord(*n),
                    base: asm::GpOrSpRegister::SP,
                    offset: asm::Imm9Signed::new(offset),
                }],
                ArmRegister::FloatingPoint(n) => vec![asm::Instruction::LoadFPUnscaled {
                    reg: asm::FPRegister::DoublePrecision(*n),
                    base: asm::GpOrSpRegister::SP,
                    offset: asm::Imm9Signed::new(offset),
                }],
            },
            |ty| match ty {
                ir::Type::I32 => (4, 4),
                ir::Type::I64 => (8, 8),
                ir::Type::Pointer(_) => (8, 8),
                ir::Type::Float => (4, 4),
                other => {
                    dbg!(&other);
                    todo!()
                }
            },
            16,
            16,
        );

        let arg_moves = {
            let starting_statements = func.block.get_statements();
            let mut statement_iter = starting_statements.into_iter();

            let mut args_moves = Vec::new();
            let arg_targets = codegen::arguments(func.arguments.iter().map(|(_, t)| t.clone()));
            for ((arg, arg_src), s) in func
                .arguments
                .iter()
                .zip(arg_targets.iter())
                .zip(statement_iter.by_ref())
            {
                let target_reg = match s {
                    ir::Statement::Assignment { target, value } => {
                        assert_eq!(arg.0, target.name);
                        assert_eq!(value, ir::Value::Unknown);

                        match register_map.get(&target).unwrap() {
                            ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                            ArmRegister::FloatingPoint(n) => todo!("Floating Point Register"),
                        }
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                };

                match arg_src {
                    ArgTarget::GPRegister(n) => {
                        args_moves.push(asm::Instruction::MovRegister {
                            dest: target_reg,
                            src: asm::GPRegister::DWord(*n),
                        });
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                };
            }

            let remaining = statement_iter.collect();
            func.block.set_statements(remaining);

            args_moves
        };

        let asm_ctx = codegen::Context {
            registers: register_map.into(),
            var: stack_allocation.var_offsets,
            pre_ret_instr: stack_allocation.pre_return_instr.clone(),
            stack_allocs: stack_allocation.allocations,
        };

        let mut asm_blocks: Vec<_> = func
            .block
            .block_iter()
            .map(|b| codegen::block_to_asm(b, &asm_ctx))
            .collect();

        {
            let first = asm_blocks.get_mut(0).unwrap();

            let mut final_instr = arg_moves;
            final_instr.extend(std::mem::take(&mut first.instructions));
            first.instructions = final_instr;
        }

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

    fn assemble(&self, input_file: &str, target_file: &str) {
        let output = Command::new("as")
            .args(["-o", target_file, input_file])
            .output()
            .expect("Failed to assemble");

        if !output.status.success() {
            let err_str = String::from_utf8(output.stderr).unwrap();
            panic!("{}", err_str);
        }
    }

    fn link(&self, files: &[&str], target_file: &str) {
        let output = Command::new("ld")
            .args(["-macosx_version_min", "12.0.0"])
            .args(["-o", target_file])
            .args(files)
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

        if !output.status.success() {
            let err_str = String::from_utf8(output.stderr).unwrap();
            panic!("{}", err_str);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ArmRegister {
    GeneralPurpose(u8),
    FloatingPoint(u8),
}

impl From<ArmRegister> for asm::Register {
    fn from(src: ArmRegister) -> Self {
        match src {
            ArmRegister::GeneralPurpose(n) => {
                asm::Register::GeneralPurpose(asm::GPRegister::DWord(n))
            }
            ArmRegister::FloatingPoint(n) => {
                asm::Register::FloatingPoint(asm::FPRegister::DoublePrecision(n))
            }
        }
    }
}

impl util::registers::Register for ArmRegister {
    fn reg_type(&self) -> util::registers::RegisterType {
        match self {
            Self::GeneralPurpose(_) => util::registers::RegisterType::GeneralPurpose,
            Self::FloatingPoint(_) => util::registers::RegisterType::FloatingPoint,
        }
    }

    fn align_size(&self) -> (usize, usize) {
        match self {
            Self::GeneralPurpose(_) => (8, 8),
            Self::FloatingPoint(_) => (8, 8),
        }
    }
}

impl Target for Backend {
    fn generate(&self, program: ir::Program) {
        let global_statements = program.global.get_statements();
        for stmnt in global_statements {
            dbg!(&stmnt);
            todo!("Generate Global stuff")
        }

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

        std::fs::write("./code.s", asm_text).unwrap();

        self.assemble("./code.s", "./code.o");
        self.link(&["./code.o"], "./code");

        std::fs::remove_file("./code.s").unwrap();
        std::fs::remove_file("./code.o").unwrap();
    }
}
