//! The ISA and general "Spec" for the SH4a Instruction-Set and some conventions used
//!
//! # Register
//! * R0-R3: Used in the Function-Call ABI
//! * R4: This Register is used for storing Jump/Branch Target Addresses
//! * R5-14: These are General Purpose Registers and can be used for whatever Purposes needed,
//! but need to be callee saved
//! * R15: The Stack-Pointer Register
//!
//! # General
//! * Big-Endian Mode

use crate::util;

pub mod assembler;

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

    pub fn register(&self) -> u8 {
        self.0
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
    /// Stores the raw u32 Bytes into the given Register
    ///
    /// This does not directly translate to any one instruction in the Architecture but rather
    /// expands to a set of instruction to accomplish the intended goal
    MovImmR {
        /// The Raw 32-Bit value that should be stores (the endiannes of the Data should not be
        /// changed beforehand and will all be taken care of by the Assembler)
        immediate: u32,
        /// The Target Register to store the Value into
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
    /// Subtracts the src2 Register from the destination Register and stores the Result back into
    /// the Destination Register
    ///
    /// Underlying Instruction: sub Rm,Rn
    Sub {
        /// The First Part and Destination Register
        dest: GeneralPurposeRegister,
        /// The Second Part
        src2: GeneralPurposeRegister,
    },
    /// Checks if left > right for signed Values and sets the T-Bit to 1 if true and 0 if false
    ///
    /// Underlying Instruction: cmp/gt Rm,Rn
    CmpGt {
        /// The Left side of the Comparison
        left: GeneralPurposeRegister,
        /// The Right side of the Comparison
        right: GeneralPurposeRegister,
    },
    /// Checks if src > 0 for signed values and sets the T-Bit to 1 if true and 0 if false
    ///
    /// Underlying Instruction: cmp/pl Rn
    CmpPl {
        /// The Register to check
        src: GeneralPurposeRegister,
    },
    /// Ors the Target Register and the src2 Register and stores the Result back into the Target
    /// Register
    ///
    /// Underlying Instruction: or Rm,Rn
    OrRR {
        target: GeneralPurposeRegister,
        src2: GeneralPurposeRegister,
    },
    /// Shifts the Target Register by the Amount specified in the shift-Register
    ///
    /// Underlying Instruction: shld Rm,Rn
    ShldRR {
        /// The Amount of bits to shift
        shift_reg: GeneralPurposeRegister,
        /// The Register that should be shifted
        target: GeneralPurposeRegister,
    },
    /// Moves the Value of the T-Bit into the given Destination Register
    MovT {
        /// The Destination Register
        dest: GeneralPurposeRegister,
    },
    /// Jumps to some Label in the Assembly
    JumpLabel {
        /// The Label to jump to
        label: String,
    },
    /// Stores the next Instruction Adddress in the PR Register and then jumps to the Address
    /// stored in the given Register
    JumpSubroutine {
        /// The Register that contains the Value of where we want to Jump
        target: GeneralPurposeRegister,
    },
    /// Jumps to the given Label in the Assembly if the T-Bit is set to 1
    BranchTrueLabel {
        /// The Target Label to jump to
        label: String,
    },
    /// Returns from the current Subroutine
    ///
    /// Underlying Instruction: rts
    Return,
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

impl register_allocation::Register for Register {
    fn reg_type(&self) -> register_allocation::RegisterType {
        match self {
            Self::GeneralPurpose(_) => register_allocation::RegisterType::GeneralPurpose,
            Self::FloatingPoint(_) => register_allocation::RegisterType::FloatingPoint,
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
