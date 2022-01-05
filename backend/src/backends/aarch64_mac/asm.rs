// https://developer.arm.com/documentation/ddi0487/latest/

#[derive(Debug, PartialEq, Clone)]
pub enum GPRegister {
    /// A single Word Register, meaning the 32-Bit Register is used
    Word(u8),
    /// A Double Word Register, meaning the 64-Bit Register is used
    DWord(u8),
}

#[derive(Debug, PartialEq, Clone)]
pub enum GpOrSpRegister {
    GP(GPRegister),
    SP,
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
    StpPreIndex {
        first: GPRegister,
        second: GPRegister,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub name: String,
    pub instructions: Vec<Instruction>,
}
