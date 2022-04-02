use super::Target;
use crate::{util, TargetConfig};

pub struct Backend {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Register {
    GeneralPurpose(u8),
    FloatingPoint(u8),
}

impl register_allocation::Register for Register {
    fn reg_type(&self) -> register_allocation::RegisterType {
        match self {
            Self::GeneralPurpose(_) => register_allocation::RegisterType::GeneralPurpose,
            Self::FloatingPoint(_) => register_allocation::RegisterType::FloatingPoint,
        }
    }

    fn align_size(&self) -> (usize, usize) {
        match self {
            Self::GeneralPurpose(_) => (8, 8),
            Self::FloatingPoint(_) => (8, 8),
        }
    }
}

impl Backend {
    pub fn new() -> Self {
        Self {}
    }

    pub fn all_registers() -> [Register; 9] {
        [
            Register::GeneralPurpose(0),
            Register::GeneralPurpose(1),
            Register::GeneralPurpose(2),
            Register::GeneralPurpose(3),
            Register::GeneralPurpose(4),
            Register::GeneralPurpose(5),
            Register::GeneralPurpose(6),
            Register::GeneralPurpose(7),
            Register::FloatingPoint(0),
        ]
    }
}

impl Target for Backend {
    fn generate(&self, program: ir::Program, conf: TargetConfig) {
        dbg!(&conf);

        let all_registers = Self::all_registers();

        for (_, func) in program.functions.iter() {
            let registers = util::registers::allocate_registers(func, &all_registers);

            let _ = registers;
        }

        todo!("Generate")
    }
}
