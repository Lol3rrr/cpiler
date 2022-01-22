mod arguments;
use std::collections::HashMap;

pub use arguments::*;

mod to_asm;
pub use to_asm::*;

mod expression;
mod statement;

pub mod util;

use super::{asm, ArmRegister};

pub struct Context {
    pub registers: RegisterMap,
    pub pre_ret_instr: Vec<asm::Instruction>,
    pub var: HashMap<String, isize>,
    pub stack_allocs: HashMap<ir::Variable, isize>,
}

pub struct RegisterMap {
    map: HashMap<ir::Variable, ArmRegister>,
}

impl From<HashMap<ir::Variable, ArmRegister>> for RegisterMap {
    fn from(map: HashMap<ir::Variable, ArmRegister>) -> Self {
        Self { map }
    }
}

impl RegisterMap {
    pub fn get_reg(&self, var: &ir::Variable) -> Option<asm::Register> {
        let raw = self.map.get(var)?;

        let result = match raw {
            ArmRegister::GeneralPurpose(n) => match &var.ty {
                ir::Type::I64 | ir::Type::U64 | ir::Type::Pointer(_) => {
                    asm::Register::GeneralPurpose(asm::GPRegister::DWord(*n))
                }
                _ => asm::Register::GeneralPurpose(asm::GPRegister::Word(*n)),
            },
            ArmRegister::FloatingPoint(n) => match &var.ty {
                ir::Type::Float => {
                    asm::Register::FloatingPoint(asm::FPRegister::SinglePrecision(*n))
                }
                ir::Type::Double | ir::Type::LongDouble => {
                    asm::Register::FloatingPoint(asm::FPRegister::DoublePrecision(*n))
                }
                _ => todo!(),
            },
        };

        Some(result)
    }
}
