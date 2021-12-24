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
}
