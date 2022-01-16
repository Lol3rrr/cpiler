use crate::{Constant, Expression, Operand, Variable};

#[derive(Clone)]
pub enum Statement<B, WB> {
    /// An Assignment of the given Value to the provided Variable-Instance
    Assignment {
        /// The Variable that the Value should be assigned to
        target: Variable,
        /// The Value that should be assigned
        value: Value<WB>,
    },
    /// This writes the Value to some location in memory, mostly done through a Pointer
    WriteMemory {
        /// The Target on where to write the Value
        target: Operand,
        /// The Value
        value: Value<WB>,
    },
    /// A single Function-Call
    Call {
        /// The Name of the Function to call
        name: String,
        /// The Arguments for the Function
        arguments: Vec<Operand>,
    },
    /// This indicates that the Variable should be saved, usually on the Stack
    SaveVariable {
        /// The Variable that should be saved
        var: Variable,
    },
    /// Some inline assembly statements that will be handled by the Backend
    InlineAsm {
        /// The ASM Template
        template: String,
        /// The Variables passed as inputs to the Template
        inputs: Vec<Variable>,
        /// The Variable passed as an output
        output: Option<Variable>,
    },
    /// Returns the given Variable from the Function
    Return(Option<Variable>),
    /// Jumps to the given Block unconditionally
    Jump(B),
    /// Jumps to the given Block if the Variable is true
    JumpTrue(Variable, B),
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
