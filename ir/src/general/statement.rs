use std::fmt::Debug;

use crate::{comp::CompareGraph, Operand, Variable};

use super::{JumpMetadata, Value};

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
        value: Operand,
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
    /// Indicates that the Global Variable should be saved, globally
    SaveGlobalVariable {
        /// The Name of the Global Variable to save
        name: String,
        /// The Variable that contains the Data to save
        value: Variable,
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
    Jump(B, JumpMetadata),
    /// Jumps to the given Block if the Variable is true
    JumpTrue(Variable, B, JumpMetadata),
}

impl<B, WB> CompareGraph for Statement<B, WB>
where
    B: CompareGraph,
    WB: PartialEq,
{
    fn compare(
        &self,
        other: &Self,
        blocks: &mut std::collections::HashMap<*const crate::InnerBlock, usize>,
        current_block: usize,
    ) -> bool {
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
            (
                Self::WriteMemory {
                    target: s_target,
                    value: s_value,
                },
                Self::WriteMemory {
                    target: o_target,
                    value: o_value,
                },
            ) => s_target == o_target && s_value == o_value,
            (
                Self::Call {
                    name: s_name,
                    arguments: s_arguments,
                },
                Self::Call {
                    name: o_name,
                    arguments: o_arguments,
                },
            ) => s_name == o_name && s_arguments == o_arguments,
            (Self::SaveVariable { var: s_var }, Self::SaveVariable { var: o_var }) => {
                s_var == o_var
            }
            (
                Self::InlineAsm {
                    template: s_temp,
                    inputs: s_in,
                    output: s_out,
                },
                Self::InlineAsm {
                    template: o_temp,
                    inputs: o_in,
                    output: o_out,
                },
            ) => s_temp == o_temp && s_in == o_in && s_out == o_out,
            (Self::Return(s_var), Self::Return(o_var)) => s_var == o_var,
            (Self::Jump(s_next, _), Self::Jump(o_next, _)) => {
                s_next.compare(o_next, blocks, current_block)
            }
            (Self::JumpTrue(s_var, s_next, _), Self::JumpTrue(o_var, o_next, _)) => {
                if s_var != o_var {
                    return false;
                }

                s_next.compare(o_next, blocks, current_block)
            }
            _ => false,
        }
    }
}

impl<B, WB> Statement<B, WB>
where
    WB: Debug,
{
    pub(crate) fn print<BP>(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        print_block: BP,
    ) -> std::fmt::Result
    where
        BP: Fn(&B) -> String,
    {
        match self {
            Self::Assignment { target, value } => f
                .debug_struct("Assignment")
                .field("target", target)
                .field("value", value)
                .finish(),
            Self::SaveVariable { var } => f.debug_struct("SaveVariable").field("var", var).finish(),
            Self::SaveGlobalVariable { name, value } => f
                .debug_struct("SaveGlobalVariable")
                .field("name", name)
                .field("value", value)
                .finish(),
            Self::WriteMemory { target, value } => f
                .debug_struct("WriteMemory")
                .field("target", target)
                .field("value", value)
                .finish(),
            Self::Call { name, arguments } => f
                .debug_struct("Call")
                .field("name", name)
                .field("arguments", arguments)
                .finish(),
            Self::InlineAsm {
                template,
                output,
                inputs,
            } => f
                .debug_struct("InlineAsm")
                .field("template", template)
                .field("output", output)
                .field("inputs", inputs)
                .finish(),
            Self::Return(var) => f.debug_tuple("Return").field(var).finish(),
            Self::Jump(target, _) => f.debug_tuple("Jump").field(&(print_block(target))).finish(),
            Self::JumpTrue(var, target, _) => f
                .debug_tuple("JumpTrue")
                .field(var)
                .field(&(print_block(target)))
                .finish(),
        }
    }
}

impl<B, WB> Statement<B, WB>
where
    WB: Clone + 'static,
{
    /// Returns a list of all the used Variables in this Statement
    ///
    /// # Note
    /// This does not contain the Targets of Assignment Statements
    pub fn used_vars(&self) -> UsedVariableIter {
        match self {
            Self::Assignment { value, .. } => value.used_vars(),
            Self::SaveVariable { var } => var.clone().into(),
            Self::SaveGlobalVariable { value, .. } => value.clone().into(),
            Self::WriteMemory { target, value } => {
                let target_iter = target.used_vars();
                let value_iter = value.used_vars();

                UsedVariableIter::VarLength(Box::new(target_iter.chain(value_iter)))
            }
            Self::Call { arguments, .. } => {
                let owned = arguments.clone();

                UsedVariableIter::VarLength(Box::new(owned.into_iter().flat_map(|a| a.used_vars())))
            }
            Self::InlineAsm { inputs, output, .. } => {
                let inputs_iter = inputs.clone().into_iter();
                let output_iter = output.clone().into_iter();

                UsedVariableIter::VarLength(Box::new(inputs_iter.chain(output_iter)))
            }
            Self::Return(None) => UsedVariableIter::Empty,
            Self::Return(Some(var)) => var.clone().into(),
            Self::Jump(_, _) => UsedVariableIter::Empty,
            Self::JumpTrue(var, _, _) => var.clone().into(),
        }
    }
}

pub enum UsedVariableIter {
    Empty,
    Single(std::iter::Once<Variable>),
    Double(std::iter::Chain<std::iter::Once<Variable>, std::iter::Once<Variable>>),
    VarLength(Box<dyn Iterator<Item = Variable>>),
}

impl Iterator for UsedVariableIter {
    type Item = Variable;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Empty => None,
            Self::Single(iter) => iter.next(),
            Self::Double(iter) => iter.next(),
            Self::VarLength(iter) => iter.next(),
        }
    }
}

impl From<Variable> for UsedVariableIter {
    fn from(other: Variable) -> Self {
        Self::Single(std::iter::once(other))
    }
}
