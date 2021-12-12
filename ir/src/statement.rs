use std::{collections::HashSet, fmt::Debug, sync::Arc};

use crate::{BasicBlock, Expression, Value, Variable};

/// A Statement in the IR contains a single "Instruction", like evaluating an expression and/or
/// storing its result in a new Variable or jumping to a different Point in the Program
#[derive(Clone)]
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
            (Self::Expression(s_exp), Self::Expression(o_exp)) => s_exp == o_exp,
            (Self::Return(s_var), Self::Return(o_var)) => s_var == o_var,
            (Self::Jump(s_next), Self::Jump(o_next)) => s_next == o_next,
            (Self::JumpTrue(s_var, s_next), Self::JumpTrue(o_var, o_next)) => {
                s_var == o_var && s_next == o_next
            }
            _ => false,
        }
    }
}

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assignment { target, value } => {
                write!(f, "Assignment({:?} = {:?})", target, value)
            }
            Self::Expression(exp) => {
                write!(f, "Expression({:?})", exp)
            }
            Self::Return(val) => write!(f, "Return({:?})", val),
            Self::Jump(target) => {
                let ptr = Arc::as_ptr(target);

                write!(f, "Jump(0x{:x})", ptr as usize)
            }
            Self::JumpTrue(var, target) => {
                let ptr = Arc::as_ptr(target);

                write!(f, "JumpTrue({:?}, 0x{:x})", var, ptr as usize)
            }
        }
    }
}

impl Statement {
    /// Generates the .dot graphviz stuff
    pub fn to_dot(
        &self,
        lines: &mut Vec<String>,
        drawn: &mut HashSet<*const BasicBlock>,
        block_ptr: *const BasicBlock,
        number: usize,
        src: &str,
    ) -> String {
        let name = format!("block_{}_s{}", block_ptr as usize, number);

        match self {
            Self::Assignment { target, value } => {
                let content = format!("{:?} = {:?}", target, value);
                let node_line = format!("{} [label = \"{}\"]", name, content.replace('"', "\\\""));
                lines.push(node_line);

                let line = format!("{} -> {}", src, name);
                lines.push(line);
            }
            Self::Expression(exp) => {
                let content = format!("{:?}", exp);
                let node_line = format!("{} [label = \"{}\"]", name, content.replace('"', "\\\""));
                lines.push(node_line);

                let line = format!("{} -> {}", src, name);
                lines.push(line);
            }
            Self::Return(val) => {
                let content = format!("return {:?}", val);
                let node_line = format!("{} [label = \"{}\"]", name, content.replace('"', "\\\""));
                lines.push(node_line);

                let line = format!("{} -> {}", src, name);
                lines.push(line);
            }
            Self::Jump(target) => {
                let content = "Jump".to_string();
                let node_line = format!("{} [label = \"{}\"]", name, content.replace('"', "\\\""));
                lines.push(node_line);

                let line = format!("{} -> {}", src, name);
                lines.push(line);

                let target_name = target.to_dot(lines, drawn);

                let target_line = format!("{} -> {}", name, target_name);
                lines.push(target_line);
            }
            Self::JumpTrue(cond, target) => {
                let content = "JumpTrue".to_string();
                let node_line = format!("{} [label = \"{}\"]", name, content.replace('"', "\\\""));
                lines.push(node_line);

                let line = format!("{} -> {}", src, name);
                lines.push(line);

                let target_name = target.to_dot(lines, drawn);

                let var_str = format!("{:?}", cond);
                let target_line = format!(
                    "{} -> {} [label = \"{}\"]",
                    name,
                    target_name,
                    var_str.replace('"', "\\\"")
                );
                lines.push(target_line);
            }
        };

        name
    }
}
