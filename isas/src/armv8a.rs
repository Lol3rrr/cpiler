// https://developer.arm.com/documentation/ddi0487/latest/

use std::fmt::Display;

mod instruction;
pub use instruction::*;

mod immediate;
pub use immediate::*;

#[derive(Debug, PartialEq, Clone)]
pub enum GPRegister {
    /// A single Word Register, meaning the 32-Bit Register is used
    Word(u8),
    /// A Double Word Register, meaning the 64-Bit Register is used
    DWord(u8),
}

impl Display for GPRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Word(n) => write!(f, "w{}", n),
            Self::DWord(n) => write!(f, "x{}", n),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FPRegister {
    SinglePrecision(u8),
    DoublePrecision(u8),
}

impl Display for FPRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SinglePrecision(n) => write!(f, "s{}", n),
            Self::DoublePrecision(n) => write!(f, "d{}", n),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Register {
    GeneralPurpose(GPRegister),
    FloatingPoint(FPRegister),
}

#[derive(Debug, PartialEq, Clone)]
pub enum GpOrSpRegister {
    GP(GPRegister),
    SP,
}

impl Display for GpOrSpRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GP(gp) => gp.fmt(f),
            Self::SP => write!(f, "sp"),
        }
    }
}

/// Page: 197
#[derive(Debug, PartialEq, Clone)]
pub enum Cond {
    Equal,
    NotEqual,
    /// Unsigned greater than or unordered
    Hi,
    /// Unsiged lower than or equal
    /// + Float less than or equal
    Ls,
    /// Signed greater than or equal
    /// + Float greater than or equal
    Ge,
    /// Signed greater than
    /// + Float greater than
    Gt,
    /// Signed less than or equal
    Le,
    /// Signed less than
    Lt,
    /// Float less than
    Mi,
}

impl Display for Cond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Equal => write!(f, "eq"),
            Self::NotEqual => write!(f, "ne"),
            Self::Hi => write!(f, "hi"),
            Self::Ls => write!(f, "ls"),
            Self::Ge => write!(f, "ge"),
            Self::Gt => write!(f, "gt"),
            Self::Le => write!(f, "le"),
            Self::Lt => write!(f, "lt"),
            Self::Mi => write!(f, "mi"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn to_text(&self) -> String {
        let mut result = String::new();

        result.push_str(&self.name);
        result.push_str(": ");

        if let Some(first_instr) = self.instructions.first() {
            let instr_str = format!("{}\n", first_instr);

            result.push_str(&instr_str);
        }

        let indent = self.name.len() + 2;
        for instr in self.instructions.iter().skip(1) {
            let instr_str = format!("{:indent$}{}\n", "", instr, indent = indent);

            result.push_str(&instr_str);
        }

        result
    }
}
