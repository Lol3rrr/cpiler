#[derive(Debug, PartialEq, Clone)]
pub enum Arch {
    X86,
    X86_64,
    AArch64,
    SH4A_FXCG50,
}

impl Arch {
    pub fn ptr_size(&self) -> usize {
        match self {
            Self::X86 => 4,
            Self::X86_64 => 8,
            Self::AArch64 => 8,
            Self::SH4A_FXCG50 => 4,
        }
    }

    pub fn ptr_type(&self) -> ir::Type {
        match self {
            Self::X86 => ir::Type::I32,
            Self::X86_64 => ir::Type::I64,
            Self::AArch64 => ir::Type::I64,
            Self::SH4A_FXCG50 => ir::Type::I32,
        }
    }

    pub fn ptr_const(&self, value: i64) -> ir::Constant {
        match self {
            Self::X86 => ir::Constant::I32(value as i32),
            Self::X86_64 => ir::Constant::I64(value),
            Self::AArch64 => ir::Constant::I64(value),
            Self::SH4A_FXCG50 => ir::Constant::I32(value as i32),
        }
    }
}
