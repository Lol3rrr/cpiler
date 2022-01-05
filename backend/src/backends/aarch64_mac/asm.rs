// https://developer.arm.com/documentation/ddi0487/latest/

mod instruction;
use std::fmt::Display;

pub use instruction::*;

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

#[derive(Debug, PartialEq, Clone)]
pub enum Cond {
    Equal,
    NotEqual,
    /// Unsigned greater than or unordered
    Hi,
    /// Unsiged lower than or equal
    Ls,
    /// Signed greater than or equal
    Ge,
    /// Signed greater than
    Gt,
    /// Signed less than or equal
    Le,
    /// Signed less than
    Lt,
}

impl Display for Cond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gt => write!(f, "ge"),
            other => {
                dbg!(&other);

                todo!()
            }
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
