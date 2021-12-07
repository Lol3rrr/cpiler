use std::sync::Weak;

use crate::{BasicBlock, Expression, Variable};

/// This holds the Information for a single Source for a PhiNode
#[derive(Debug, Clone)]
pub struct PhiEntry {
    /// The Block in which this Variable definition can be found
    pub block: Weak<BasicBlock>,
    /// The Variable found
    pub var: Variable,
}

impl PartialEq for PhiEntry {
    fn eq(&self, other: &Self) -> bool {
        self.block.as_ptr() == other.block.as_ptr() && self.var == other.var
    }
}

/// A Value that will be assigned to a Variable
#[derive(Debug, Clone)]
pub enum Value {
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
        sources: Vec<PhiEntry>,
    },
    /// The Value can be obtained from the given Expression
    Expression(Expression),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unknown, Self::Unknown) => true,
            (Self::Constant(s_c), Self::Constant(o_c)) => s_c == o_c,
            (Self::Phi { sources: s_sources }, Self::Phi { sources: o_sources }) => {
                s_sources == o_sources
            }
            (Self::Expression(s_exp), Self::Expression(o_exp)) => s_exp == o_exp,
            _ => false,
        }
    }
}

/// A Constant Value that is already known at compile-time
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    /// 8 bit signed integer
    I8(i8),
    /// 16 bit signed integer
    I16(i16),
    /// 32 bit signed integer
    I32(i32),
    /// 64 bit signed integer
    I64(i64),
    /// 8 bit unsigned integer
    U8(u8),
    /// 16 bit unsigned integer
    U16(u16),
    /// 32 bit unsigned integer
    U32(u32),
    /// 64 bit unsigned integer
    U64(u64),
}
