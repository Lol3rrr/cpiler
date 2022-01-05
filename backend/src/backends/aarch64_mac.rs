// https://developer.apple.com/documentation/xcode/writing-arm64-code-for-apple-platforms
// https://developer.arm.com/documentation/102374/0101/Registers-in-AArch64---general-purpose-registers

use std::collections::HashMap;

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
    fn registers() -> [ArmRegister; 39] {
        [
            //ArmRegister::GeneralPurpose(8),
            ArmRegister::GeneralPurpose(9),
            ArmRegister::GeneralPurpose(10),
            ArmRegister::GeneralPurpose(11),
            ArmRegister::GeneralPurpose(12),
            ArmRegister::GeneralPurpose(13),
            ArmRegister::GeneralPurpose(14),
            ArmRegister::GeneralPurpose(15),
            //ArmRegister::GeneralPurpose(16),
            //ArmRegister::GeneralPurpose(17),
            //ArmRegister::GeneralPurpose(18),
            /*
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
            */
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

    fn codegen(&self, func: &ir::FunctionDefinition, register_map: HashMap<Variable, ArmRegister>) {
        let arg_targets = codegen::arguments(func.arguments.iter().map(|(_, t)| t.clone()));
        dbg!(&func.arguments, &arg_targets);

        for (arg, arg_register) in func.arguments.iter().zip(arg_targets.iter()) {
            dbg!(&arg, &arg_register);
            todo!()
        }

        let stack_space = codegen::stack_space(func);
        dbg!(&stack_space);

        // TODO
        // Allocate space for all the Variables that need to be on the Stack

        let stack_setup = asm::Instruction::StpPreIndex {
            first: asm::GPRegister::DWord(29),
            second: asm::GPRegister::DWord(30),
            base: asm::GpOrSpRegister::SP,
            offset: -(stack_space as i16),
        };
        dbg!(&stack_setup);

        let pre_return_instr = vec![asm::Instruction::LdpPostIndex {
            first: asm::GPRegister::DWord(29),
            second: asm::GPRegister::DWord(30),
            base: asm::GpOrSpRegister::SP,
            offset: stack_space as i16,
        }];

        let mut asm_blocks: Vec<_> = func
            .block
            .block_iter()
            .map(|b| codegen::block_to_asm(b, &register_map, pre_return_instr.clone()))
            .collect();

        asm_blocks
            .first_mut()
            .unwrap()
            .instructions
            .insert(0, stack_setup);

        dbg!(&asm_blocks);

        todo!("Codegen for Function")
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
        for (_, func) in program.functions.iter() {
            dbg!(&func);

            let registers = util::registers::allocate_registers(func, &all_registers);

            dbg!(&registers);

            util::destructure::destructure_func(func);

            self.codegen(func, registers);
        }

        todo!()
    }
}
