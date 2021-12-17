use crate::{Constant, Type, Variable};

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
    /// A Call to some function
    FunctionCall {
        /// The Name of the function to call
        name: String,
        /// The given Arguments for this Call
        arguments: Vec<Operand>,
        /// The returned Type
        return_ty: Type,
    },
}
