use std::collections::HashMap;

use super::Target;
use crate::{util, TargetConfig};

use crate::isas::x86;

pub struct Backend {}

impl Backend {
    pub fn new() -> Self {
        Self {}
    }

    pub fn all_registers() -> [x86::Register; 9] {
        [
            x86::Register::GeneralPurpose(x86::GeneralPurposeRegister::Double(0)),
            x86::Register::GeneralPurpose(x86::GeneralPurposeRegister::Double(1)),
            x86::Register::GeneralPurpose(x86::GeneralPurposeRegister::Double(2)),
            x86::Register::GeneralPurpose(x86::GeneralPurposeRegister::Double(3)),
            x86::Register::GeneralPurpose(x86::GeneralPurposeRegister::Double(4)),
            x86::Register::GeneralPurpose(x86::GeneralPurposeRegister::Double(5)),
            x86::Register::GeneralPurpose(x86::GeneralPurposeRegister::Double(6)),
            x86::Register::GeneralPurpose(x86::GeneralPurposeRegister::Double(7)),
            x86::Register::FloatingPoint,
        ]
    }

    fn codegen(
        &self,
        _func: &ir::FunctionDefinition,
        _registers: HashMap<ir::Variable, x86::Register>,
    ) -> Vec<x86::Block> {
        todo!("Codegen")
    }
}

impl Target for Backend {
    fn generate(&self, program: ir::Program, conf: TargetConfig) {
        dbg!(&conf);

        let all_registers = Self::all_registers();

        let mut blocks = Vec::new();
        for (_, func) in program.functions.iter() {
            let registers = util::registers::allocate_registers(
                func,
                &all_registers,
                Some(conf.build_dir.clone()),
            );

            util::destructure::destructure_func(func);

            let func_blocks = self.codegen(func, registers);
            blocks.extend(func_blocks);
        }

        todo!("Generate")
    }
}
