#![warn(missing_docs)]
//! # General
//! This IR is in the SSA-Form and in general is designed to be fairly easy to use and understand.
//! To accomplish this, there are a couple of factors:
//!
//! ## No nested Expression
//! This means that the Operands for a given Expression can only be a Variable or a Constant. This
//! will cause more Statements to be emitted in the IR as all the nested Expressions need to be
//! broken up into smaller pieces and need to be stored in temporary Variables. However this makes
//! the optimizations easier to implement down the Line and also allows for easier Code-Generation
//! in the End because they are already in mostly the right format for it to be translated more or
//! less directly.
//!
//! ## Only Tracking at Scalar-Variable level
//! This means that in cases where we have a Pointer, Array or Struct it treats any modification of
//! the underlying Data or any of its Members is seen as a modification of the Variable itself.
//! This simplifies the overall Structure as we dont need to track any extra Data depending on what
//! type of Variable it is, but also means that we lost some optimization opportunities and also
//! likely produce less efficient code as we have to reread them more often

use std::sync::Arc;

mod variable;
pub use variable::*;

mod ty;
pub use ty::*;

mod value;
pub use value::*;

mod expression;
pub use expression::*;

mod block;
pub use block::*;

/// The overall Program Structure that contains all the needed information about the Program itself
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// This contains definitions for Global Variables that need to be usable by the function
    /// definitions
    pub global: Arc<BasicBlock>,
    /// The various Function Definitions in the Program
    pub functions: Vec<FunctionDefinition>,
}

/// A Statement in the IR contains a single "Instruction", like evaluating an expression and/or
/// storing its result in a new Variable or jumping to a different Point in the Program
#[derive(Debug, Clone)]
pub enum Statement {
    /// An Assignment of the given Value to the provided Variable-Instance
    Assignment {
        /// The Variable that the Value should be assigned to
        target: Variable,
        /// The Value that should be assigned
        value: Value,
    },
    /// A single Expression that does not modify any of the Variables
    Expression(Expression),
    /// Returns the given Variable from the Function
    Return(Option<Variable>),
    /// Jumps to the given Block unconditionally
    Jump(Arc<BasicBlock>),
    /// Jumps to the given Block if the Variable is true
    JumpTrue(Variable, Arc<BasicBlock>),
}

impl PartialEq for Statement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Assignment {
                    target: s_target,
                    value: s_value,
                },
                Self::Assignment {
                    target: o_target,
                    value: o_value,
                },
            ) => s_target == o_target && s_value == o_value,
            (Self::Expression(s_exp), Self::Expression(o_exp)) => {
                todo!()
            }
            (Self::Return(s_var), Self::Return(o_var)) => s_var == o_var,
            (Self::Jump(s_next), Self::Jump(o_next)) => s_next == o_next,
            (Self::JumpTrue(s_var, s_next), Self::JumpTrue(o_var, o_next)) => {
                s_var == o_var && s_next == o_next
            }
            _ => false,
        }
    }
}

/// A definition of a Function
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    /// The Arguments of the Function in the Order they will be received in
    pub arguments: Vec<(String, Type)>,
    /// The initial BasicBlock of the Function
    pub block: Arc<BasicBlock>,
    /// The Return Type of the Function
    pub return_ty: Type,
}
