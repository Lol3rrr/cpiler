use std::fmt::Display;

use super::{Cond, GPRegister, GpOrSpRegister};

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Nop,
    /// Moves the Value from the given Regsiter into the StackPointer Register
    MovToSP {
        src: GPRegister,
    },
    /// Moves the Value of the StackPointer Register in to the dest Register
    MovFromSP {
        dest: GPRegister,
    },
    /// Moves/Copies the value from the src Register to the dest Register
    MovRegister {
        dest: GPRegister,
        src: GPRegister,
    },
    /// Moves the unsigned immediate into the given Register and optionally shift it
    Movz {
        dest: GPRegister,
        shift: u8,
        immediate: u16,
    },
    AddCarry {
        dest: GPRegister,
        src1: GPRegister,
        src2: GPRegister,
    },
    AddImmediate {
        dest: GPRegister,
        src: GPRegister,
        immediate: u16,
        shift: u8,
    },
    SubImmediate {
        dest: GPRegister,
        src: GPRegister,
        immediate: u16,
        shift: u8,
    },
    /// Page: 1434
    StoreRegisterUnscaled {
        reg: GPRegister,
        base: GpOrSpRegister,
        offset: i16,
    },
    StpPreIndex {
        first: GPRegister,
        second: GPRegister,
        base: GpOrSpRegister,
        offset: i16,
    },
    /// Page: 1190
    LoadRegisterUnscaled {
        reg: GPRegister,
        base: GpOrSpRegister,
        offset: i16,
    },
    LdpPostIndex {
        first: GPRegister,
        second: GPRegister,
        base: GpOrSpRegister,
        offset: i16,
    },
    JumpLabel {
        target: String,
    },
    CmpImmediate {
        reg: GPRegister,
        immediate: u16,
        shift: u8,
    },
    /// Set the Target register to 1 if the Condition is true or to 0 if it is false
    CSet {
        target: GPRegister,
        condition: Cond,
    },
    /// Checks if the Value of the Register is Non Zero and then branches to the Target
    BranchNonZeroLabel {
        reg: GPRegister,
        target: String,
    },
    BranchLabelCond {
        target: String,
        condition: Cond,
    },
    Call {
        target: String,
    },
    Return,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StoreRegisterUnscaled { reg, base, offset } => {
                write!(f, "stur {}, [{}, #{}]", reg, base, offset)
            }
            Self::StpPreIndex {
                first,
                second,
                base,
                offset,
            } => {
                write!(f, "stp {}, {}, [{}, #{}]!", first, second, base, offset)
            }
            Self::LoadRegisterUnscaled { reg, base, offset } => {
                write!(f, "ldur {}, [{}, #{}]", reg, base, offset)
            }
            Self::LdpPostIndex {
                first,
                second,
                base,
                offset,
            } => {
                write!(f, "ldp {}, {}, [{}], #{}", first, second, base, offset)
            }
            Self::Movz {
                dest,
                immediate,
                shift,
            } => {
                write!(f, "movz {}, #{}, LSL #{}", dest, immediate, shift)
            }
            Self::MovRegister { dest, src } => {
                write!(f, "mov {}, {}", dest, src)
            }
            Self::SubImmediate {
                dest,
                src,
                immediate,
                shift,
            } => {
                write!(f, "sub {}, {}, #{}, LSL #{}", dest, src, immediate, shift)
            }
            Self::CmpImmediate {
                reg,
                immediate,
                shift,
            } => {
                write!(f, "cmp {}, #{}, LSL #{}", reg, immediate, shift)
            }
            Self::CSet { target, condition } => {
                write!(f, "cset {}, {}", target, condition)
            }
            Self::JumpLabel { target } => {
                write!(f, "b {}", target)
            }
            Self::BranchNonZeroLabel { reg, target } => {
                write!(f, "cbnz {}, {}", reg, target)
            }
            Self::Return => {
                write!(f, "ret")
            }
            other => {
                dbg!(&other);
                todo!()
            }
        }
    }
}
