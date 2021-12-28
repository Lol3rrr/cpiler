#[derive(Debug, PartialEq, Clone)]
pub enum Arch {
    X86,
    X86_64,
    AArch64,
}

impl Arch {
    pub fn ptr_size(&self) -> usize {
        match self {
            Self::X86 => 4,
            Self::X86_64 => 8,
            Self::AArch64 => 8,
        }
    }

    pub fn ptr_type(&self) -> ir::Type {
        match self {
            Self::X86 => ir::Type::I32,
            Self::X86_64 => ir::Type::I64,
            Self::AArch64 => ir::Type::I64,
        }
    }

    pub fn ptr_const(&self, value: i64) -> ir::Constant {
        match self {
            Self::X86 => ir::Constant::I32(value as i32),
            Self::X86_64 => ir::Constant::I64(value),
            Self::AArch64 => ir::Constant::I64(value),
        }
    }
}
