use crate::util;

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    name: String,
    instructions: Vec<Instruction>,
}

impl Block {
    pub fn new(name: String, instructions: Vec<Instruction>) -> Self {
        Self { name, instructions }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct GeneralPurposeRegister(u8);

impl GeneralPurposeRegister {
    pub fn new(numb: u8) -> Self {
        if numb > 15 {
            panic!()
        }

        Self(numb)
    }

    pub fn stack_reg() -> Self {
        Self(15)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    /// No Operation
    Nop,
    /// Copies the Value from the src Register into the dest Register
    MovRR {
        /// The src GeneralPurpose Register
        src: GeneralPurposeRegister,
        /// The dest GeneralPurpose Register
        dest: GeneralPurposeRegister,
    },
    /// Moves the sign extended Immediate Value into the given Register
    MovIR {
        /// The immediate Value
        immediate: i8,
        /// The Register to store the Value into
        dest: GeneralPurposeRegister,
    },
    /// Calculates an Address by adding R0 and the Base-Register together, then loads a 32-Bit
    /// Value from that new Address and stores it into the Target Register
    ///
    /// Underlying Instruction: mov.l @(R0, Rm),Rn
    MovLR0PRR {
        /// The Base Register that is added together with R0
        base: GeneralPurposeRegister,
        /// The Register to store the Value in
        target: GeneralPurposeRegister,
    },
    /// Calculates an Address by adding R0 and the Base-Register together, then writes the 32-Bit
    /// Value from the src-Register at the Target Address
    ///
    /// Underlying Instruction: mov.l Rm,@(R0,Rn)
    MovLRR0PR {
        /// The Base Register that is added together with R0
        base: GeneralPurposeRegister,
        /// The Register whose Value will be written to Memory
        src: GeneralPurposeRegister,
    },
    /// Moves the Value from the Register into the Procedure Register
    ///
    /// Underlying Instruction: lds Rm,PR
    MovRPR {
        /// The Register whose Value should be copied to the Procedure Register
        src: GeneralPurposeRegister,
    },
    /// Subtracts 4 from the Stack-Pointer, making enough Room for a 32-Bit Value on it, then
    /// writes the 32-Bit Registers Value into that allocated Space
    ///
    /// Note:
    /// This is actually implemented as the mov.l Rm, @-Rn Instruction with the 15th GP Register
    /// for the Stack-Pointer
    PushL {
        /// The Register to save on the Stack
        reg: GeneralPurposeRegister,
    },
    /// Pushes the Procedure Register onto the Stack by first moving the Stack-Pointer by 4 Byte
    /// and then writes the Procedure Register into the new Address of the Stack-Pointer
    ///
    /// Underlying Instruction: sts.l PR,@-Rn
    PushPR,
    /// Copies a 32-Bit Value from the Top of the Stack into the given Register and then adds 4 to
    /// the Stack-Pointer to "free" that area again.
    ///
    /// Note:
    /// This is actually implemented as the mov.l @Rm+, Rn Instruction wit the 15th GP Register
    /// for the Stack-Pointer
    PopL {
        /// The Register to save the popped Value to
        reg: GeneralPurposeRegister,
    },
    /// Pops a 32-Bit Value from the Top of the Stack and stores it in the Procedure Register and
    /// then adds 4 to the Stack-Pointer to "free" that area again.
    ///
    /// Underlying Instruction: lds.l @Rm+,PR
    PopPR,
    /// Adds the immediate to the given Register and stores the Result back into the same Register
    AddImmediate {
        /// The base and destination Register
        reg: GeneralPurposeRegister,
        /// The Immediate Value to add
        immediate: i8,
    },
}

/// Special Registers:
/// * R15 is the Stack Pointer
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Register {
    GeneralPurpose(GeneralPurposeRegister),
    FloatingPoint(u8),
    /// The Procedure Register which stores the return Address of the current Subroutine
    PR,
}

impl Register {
    pub fn stack_ptr() -> Self {
        Self::GeneralPurpose(GeneralPurposeRegister::stack_reg())
    }
}

impl util::registers::Register for Register {
    fn reg_type(&self) -> util::registers::RegisterType {
        match self {
            Self::GeneralPurpose(_) => util::registers::RegisterType::GeneralPurpose,
            Self::FloatingPoint(_) => util::registers::RegisterType::FloatingPoint,
            Self::PR => unreachable!("This should never be considered for Register Allocation"),
        }
    }

    fn align_size(&self) -> (usize, usize) {
        match self {
            Self::GeneralPurpose(_) => (4, 4),
            other => {
                dbg!(&other);
                todo!()
            }
        }
    }
}
