use std::fmt::Debug;

use crate::{comp::CompareGraph, Operand, Variable};

use super::Value;

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
    /// Indicates that the Global Variable should be saved, globally
    SaveGlobalVariable {
        /// The Variable to save globally
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
            (Self::Jump(s_next), Self::Jump(o_next)) => {
                s_next.compare(o_next, blocks, current_block)
            }
            (Self::JumpTrue(s_var, s_next), Self::JumpTrue(o_var, o_next)) => {
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
            Self::SaveGlobalVariable { var } => f
                .debug_struct("SaveGlobalVariable")
                .field("var", var)
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
            Self::Jump(target) => f.debug_tuple("Jump").field(&(print_block(target))).finish(),
            Self::JumpTrue(var, target) => f
                .debug_tuple("JumpTrue")
                .field(var)
                .field(&(print_block(target)))
                .finish(),
        }
    }
}

impl<B, WB> Statement<B, WB> {
    /// Returns a list of all the used Variables in this Statement
    ///
    /// # Note
    /// This does not contain the Targets of Assignment Statements
    pub fn used_vars(&self) -> Vec<Variable> {
        match self {
            Self::Assignment { value, .. } => value.used_vars(),
            Self::SaveVariable { var } => vec![var.clone()],
            Self::SaveGlobalVariable { var } => vec![var.clone()],
            Self::WriteMemory { target, value } => {
                let mut tmp = target.used_vars();
                tmp.extend(value.used_vars());
                tmp
            }
            Self::Call { arguments, .. } => {
                let mut tmp = Vec::new();
                for arg in arguments {
                    tmp.extend(arg.used_vars());
                }
                tmp
            }
            Self::InlineAsm { inputs, output, .. } => {
                let mut result = Vec::new();

                result.extend(inputs.clone());
                if let Some(out) = output {
                    result.push(out.clone());
                }

                result
            }
            Self::Return(None) => Vec::new(),
            Self::Return(Some(var)) => vec![var.clone()],
            Self::Jump(_) => Vec::new(),
            Self::JumpTrue(var, _) => vec![var.clone()],
        }
    }
}
