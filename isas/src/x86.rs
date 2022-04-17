pub struct Block {
    pub name: String,
    pub content: Vec<Instruction>,
}

pub enum Instruction {
    Move {
        src: GeneralPurposeRegister,
        dest: GeneralPurposeRegister,
    },
    Add {
        dest: GeneralPurposeRegister,
        other: GeneralPurposeRegister,
    },
    Sub {
        dest: GeneralPurposeRegister,
        rhs: GeneralPurposeRegister,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeneralPurposeRegister {
    /// 16 Bit
    Short(u8),
    /// 32 Bit
    Word(u8),
    /// 64 Bit
    Double(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Register {
    GeneralPurpose(GeneralPurposeRegister),
    FloatingPoint,
}

impl register_allocation::Register for Register {
    fn reg_type(&self) -> register_allocation::RegisterType {
        match self {
            Self::GeneralPurpose(_) => register_allocation::RegisterType::GeneralPurpose,
            Self::FloatingPoint => register_allocation::RegisterType::FloatingPoint,
        }
    }

    fn align_size(&self) -> (usize, usize) {
        match self {
            Self::GeneralPurpose(_) => (8, 8),
            Self::FloatingPoint => (8, 8),
        }
    }
}
