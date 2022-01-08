use std::fmt::Display;

use super::{Cond, FPRegister, FloatImm8, GPRegister, GpOrSpRegister, Imm9Signed};

#[derive(Debug, PartialEq, Clone)]
pub enum Shift {
    LSL,
    LSR,
    ASR,
}

impl Display for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LSL => write!(f, "LSL"),
            Self::LSR => write!(f, "LSR"),
            Self::ASR => write!(f, "ASR"),
        }
    }
}

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
    /// Page: 1819
    FMovRegister {
        dest: FPRegister,
        src: FPRegister,
    },
    /// Moves the unsigned immediate into the given Register and optionally shift it
    Movz {
        dest: GPRegister,
        shift: u8,
        immediate: u16,
    },
    /// Page: 1824
    FMovImmediate {
        dest: FPRegister,
        imm: FloatImm8,
    },
    AddCarry {
        dest: GPRegister,
        src1: GPRegister,
        src2: GPRegister,
    },
    /// Page: 883
    AddImmediate {
        dest: GPRegister,
        src: GpOrSpRegister,
        immediate: u16,
        shift: u8,
    },
    /// Page: 885
    AddRegisterShifted {
        dest: GPRegister,
        src1: GPRegister,
        src2: GPRegister,
        shift: Shift,
        amount: u8,
    },
    /// Page: 1630
    FPAdd {
        dest: FPRegister,
        src1: FPRegister,
        src2: FPRegister,
    },
    SubImmediate {
        dest: GPRegister,
        src: GPRegister,
        immediate: u16,
        shift: u8,
    },
    /// Page: 1457
    SubRegisterShifted {
        dest: GPRegister,
        src1: GPRegister,
        src2: GPRegister,
        shift: Shift,
        amount: u8,
    },
    /// Page: 1917
    FPSub {
        dest: FPRegister,
        src1: FPRegister,
        src2: FPRegister,
    },
    /// Page: 1243
    MultiplyRegister {
        dest: GPRegister,
        src1: GPRegister,
        src2: GPRegister,
    },
    /// Page: 1434
    StoreRegisterUnscaled {
        reg: GPRegister,
        base: GpOrSpRegister,
        offset: Imm9Signed,
    },
    /// Page: 2290
    StoreFPUnscaled {
        reg: FPRegister,
        base: GpOrSpRegister,
        offset: Imm9Signed,
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
        offset: Imm9Signed,
    },
    /// Page: 1198
    LoadSignedWordUnscaled {
        reg: GPRegister,
        base: GpOrSpRegister,
        offset: Imm9Signed,
    },
    /// Page: 1193
    LoadHalfWordUnscaled {
        reg: GPRegister,
        base: GpOrSpRegister,
        offset: Imm9Signed,
    },
    /// Page: 1196
    LoadSignedHalfWordUnscaled {
        reg: GPRegister,
        base: GpOrSpRegister,
        offset: Imm9Signed,
    },
    /// Page: 1192
    LoadByteUnscaled {
        reg: GPRegister,
        base: GpOrSpRegister,
        offset: Imm9Signed,
    },
    /// Page: 1194
    LoadSignedByteUnscaled {
        reg: GPRegister,
        base: GpOrSpRegister,
        offset: Imm9Signed,
    },
    /// Page: 1979
    LoadFPUnscaled {
        reg: FPRegister,
        base: GpOrSpRegister,
        offset: Imm9Signed,
    },
    LdpPostIndex {
        first: GPRegister,
        second: GPRegister,
        base: GpOrSpRegister,
        offset: i16,
    },
    /// Page: 2064
    SignedIntegerToFloatingPoint {
        src: GPRegister,
        dest: FPRegister,
    },
    /// Page: 1698
    FloatingPointToSignedIntegerMinusInf {
        src: FPRegister,
        dest: GPRegister,
    },
    JumpLabel {
        target: String,
    },
    CmpImmediate {
        reg: GPRegister,
        immediate: u16,
        shift: u8,
    },
    /// Page: 984
    CmpShifted {
        first: GPRegister,
        second: GPRegister,
        shift: Shift,
        amount: u8,
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
    /// Page: 934
    BranchLinkLabel {
        target: String,
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
            Self::StoreFPUnscaled { reg, base, offset } => {
                write!(f, "str {}, [{}, #{}]", reg, base, offset)
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
            Self::LoadSignedWordUnscaled { reg, base, offset } => {
                write!(f, "ldursw {}, [{}, #{}]", reg, base, offset)
            }
            Self::LoadFPUnscaled { reg, base, offset } => {
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
            Self::FMovImmediate { dest, imm } => {
                write!(f, "fmov {}, #{}", dest, imm)
            }
            Self::FMovRegister { dest, src } => {
                write!(f, "fmov {}, {}", dest, src)
            }
            Self::AddImmediate {
                dest,
                src,
                immediate,
                shift,
            } => {
                write!(f, "add {}, {}, #{}, LSL #{}", dest, src, immediate, shift)
            }
            Self::AddRegisterShifted {
                dest,
                src1,
                src2,
                shift,
                amount,
            } => {
                write!(f, "add {}, {}, {}, {} #{}", dest, src1, src2, shift, amount)
            }
            Self::FPAdd { dest, src1, src2 } => {
                write!(f, "fadd {}, {}, {}", dest, src1, src2)
            }
            Self::SubImmediate {
                dest,
                src,
                immediate,
                shift,
            } => {
                write!(f, "sub {}, {}, #{}, LSL #{}", dest, src, immediate, shift)
            }
            Self::SubRegisterShifted {
                dest,
                src1,
                src2,
                shift,
                amount,
            } => {
                write!(f, "sub {}, {}, {}, {} #{}", dest, src1, src2, shift, amount)
            }
            Self::FPSub { dest, src1, src2 } => {
                write!(f, "fsub {}, {}, {}", dest, src1, src2)
            }
            Self::MultiplyRegister { dest, src1, src2 } => {
                write!(f, "mul {}, {}, {}", dest, src1, src2)
            }
            Self::SignedIntegerToFloatingPoint { dest, src } => {
                write!(f, "scvtf {}, {}", dest, src)
            }
            Self::FloatingPointToSignedIntegerMinusInf { dest, src } => {
                write!(f, "fcvtms {}, {}", dest, src)
            }
            Self::CmpImmediate {
                reg,
                immediate,
                shift,
            } => {
                write!(f, "cmp {}, #{}, LSL #{}", reg, immediate, shift)
            }
            Self::CmpShifted {
                first,
                second,
                shift,
                amount,
            } => {
                write!(f, "cmp {}, {}, {} #{}", first, second, shift, amount)
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
            Self::BranchLinkLabel { target } => {
                write!(f, "bl {}", target)
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
