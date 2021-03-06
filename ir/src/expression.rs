use crate::{general::UsedVariableIter, Constant, Type, Variable};

mod unary;
pub use unary::*;

mod binary;
pub use binary::*;

/// Operands are used to perform different Operations on within an Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    /// Use this Variable for calculating the Result
    Variable(Variable),
    /// Use a Constant Value
    Constant(Constant),
}

impl Operand {
    /// The result Type of the Operand
    pub fn ty(&self) -> Type {
        match self {
            Self::Variable(var) => var.ty.clone(),
            Self::Constant(con) => con.ty(),
        }
    }

    /// Gets a list of Variables used by this Operand
    pub fn used_vars(&self) -> UsedVariableIter {
        match self {
            Self::Variable(var) => UsedVariableIter::Single(std::iter::once(var.clone())),
            Self::Constant(_) => UsedVariableIter::Empty,
        }
    }
}

/// A simple Expression that performs some form of Operation on one or more Operands
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// An Operation that is applied to two Operands
    BinaryOp {
        /// The Operation to be performed
        op: BinaryOp,
        /// The Left-Hand-Side of the Operation
        left: Operand,
        /// The Right-Hand-Side of the Operation
        right: Operand,
    },
    /// An Operation that is applied to only a single Operand
    UnaryOp {
        /// The Operation to be performed
        op: UnaryOp,
        /// The Operand to perform this Operation on
        base: Operand,
    },
    /// Converts the given Operand to the corresponding Value of another Type
    Cast {
        /// The Operand to convert
        base: Operand,
        /// The Type to which the Operand should be converted
        target: Type,
    },
    /// Obtains the Address of the given Base Operand
    AdressOf {
        /// The Operand to get the Address of
        base: Operand,
    },
    /// Reads the Value from some Address in Memory
    ReadMemory {
        /// The Address to read from
        address: Operand,
        /// The Type to read
        read_ty: Type,
    },
    /// Reads the Value from the Global Variable
    ReadGlobalVariable {
        /// The unique Name of the Global-Variable to read
        name: String,
    },
    /// A Call to some function
    FunctionCall {
        /// The Name of the function to call
        name: String,
        /// The given Arguments for this Call
        arguments: Vec<Operand>,
        /// The returned Type
        return_ty: Type,
    },
    /// Allocates size bytes on the Stack and evalutes to the start Address of this Block, which
    /// should be used for reserving the Space for Arrays or Structs
    StackAlloc {
        /// The Size to allocate in Bytes
        size: usize,
        /// The Alignment of the Bytes to allocate
        alignment: usize,
    },
}

impl Expression {
    /// Gets a list of Variables that are used by this Expression
    pub fn used_vars(&self) -> UsedVariableIter {
        match self {
            Self::BinaryOp { left, right, .. } => {
                let left_iter = left.used_vars();
                let right_iter = right.used_vars();

                UsedVariableIter::VarLength(Box::new(left_iter.chain(right_iter)))
            }
            Self::UnaryOp { base, .. } => base.used_vars(),
            Self::Cast { base, .. } => base.used_vars(),
            Self::AdressOf { base } => base.used_vars(),
            Self::ReadMemory { address, .. } => address.used_vars(),
            Self::ReadGlobalVariable { .. } => UsedVariableIter::Empty,
            Self::FunctionCall { arguments, .. } => {
                let owned = arguments.clone();

                UsedVariableIter::VarLength(Box::new(owned.into_iter().flat_map(|a| a.used_vars())))
            }
            Self::StackAlloc { .. } => UsedVariableIter::Empty,
        }
    }

    /// Returns the Type of the Expression
    pub fn ty(&self) -> Type {
        match self {
            Self::BinaryOp { left, .. } => left.ty(),
            Self::UnaryOp { base, .. } => base.ty(),
            Self::Cast { target, .. } => target.clone(),
            other => {
                dbg!(other);

                todo!()
            }
        }
    }
}
