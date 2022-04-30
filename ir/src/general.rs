use std::fmt::Debug;

use crate::{Constant, Expression, Variable};

mod statement;
pub use statement::*;

/// Some Metadata about a Jump to help the Compiler better understand the Reason for this Jump and
/// aid in certain Tasks
#[derive(Debug, PartialEq, Clone)]
pub enum JumpMetadata {
    /// A Linear Jump between two Blocks
    Linear,
    /// A Branch
    Branch,
    /// A Jump into a loop, either from the start or from the end back to the start
    Loop,
    /// A Jump out of a loop
    LoopBreak,
}

/// This holds the Information for a single Source for a PhiNode
#[derive(Debug, Clone)]
pub struct PhiEntry<WB> {
    /// The Block in which this Variable definition can be found
    pub block: WB,
    /// The Variable found
    pub var: Variable,
}

impl<WB> PartialEq for PhiEntry<WB> {
    fn eq(&self, other: &Self) -> bool {
        self.var == other.var
    }
}

/// A Value that will be assigned to a Variable
#[derive(Debug, Clone)]
pub enum Value<WB> {
    /// The Value is unknown at compile-time, like arguments for a Function
    Unknown,
    /// The Value is a constant and known at compile-time
    Constant(Constant),
    /// The Value is the same as the Value of the Variable
    Variable(Variable),
    /// The Value is one of the Variables depending on which BasicBlock we came to this Point in
    /// the Program
    Phi {
        /// The Different sources for the Value of this Value
        sources: Vec<PhiEntry<WB>>,
    },
    /// The Value can be obtained from the given Expression
    Expression(Expression),
}

impl<WB> PartialEq for Value<WB> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unknown, Self::Unknown) => true,
            (Self::Constant(s_c), Self::Constant(o_c)) => s_c == o_c,
            (Self::Variable(s_var), Self::Variable(o_var)) => s_var == o_var,
            (Self::Phi { sources: s_sources }, Self::Phi { sources: o_sources }) => {
                s_sources == o_sources
            }
            (Self::Expression(s_exp), Self::Expression(o_exp)) => s_exp == o_exp,
            _ => false,
        }
    }
}
impl<WB> Value<WB>
where
    WB: Clone + 'static,
{
    /// Returns a list of all the Variables used by this Value
    pub fn used_vars(&self) -> UsedVariableIter {
        match self {
            Self::Unknown => UsedVariableIter::Empty,
            Self::Constant(_) => UsedVariableIter::Empty,
            Self::Expression(exp) => exp.used_vars(),
            Self::Variable(var) => UsedVariableIter::Single(std::iter::once(var.clone())),
            Self::Phi { sources } => {
                let owned: Vec<_> = sources.clone();
                UsedVariableIter::VarLength(Box::new(owned.into_iter().map(|entry| entry.var)))
            }
        }
    }
}
