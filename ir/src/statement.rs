use std::fmt::Debug;

use crate::{
    comp::CompareGraph,
    dot::{Context, DrawnBlocks, Lines},
    BasicBlock, Expression, ToDot, Value, Variable,
};

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
    Jump(BasicBlock),
    /// Jumps to the given Block if the Variable is true
    JumpTrue(Variable, BasicBlock),
}

impl CompareGraph for Statement {
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
            (Self::Expression(s_exp), Self::Expression(o_exp)) => s_exp == o_exp,
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
                let ptr = target.as_ptr();

                write!(f, "Jump(0x{:x})", ptr as usize)
            }
            Self::JumpTrue(var, target) => {
                let ptr = target.as_ptr();

                write!(f, "JumpTrue({:?}, 0x{:x})", var, ptr as usize)
            }
        }
    }
}

impl ToDot for Statement {
    fn to_dot(&self, lines: &mut Lines, drawn: &mut DrawnBlocks, ctx: &Context) -> String {
        let block_ptr = *ctx
            .get("block_ptr")
            .expect("")
            .downcast_ref::<usize>()
            .expect("");
        let number_in_block = *ctx
            .get("block_number")
            .expect("")
            .downcast_ref::<usize>()
            .expect("");
        let src = ctx
            .get("block_src")
            .expect("")
            .downcast_ref::<String>()
            .expect("");

        let name = format!("block_{}_s{}", block_ptr, number_in_block);

        match self {
            Self::Assignment { target, value } => {
                let content = format!("{:?} = {:?}", target, value);
                let node_line = format!("{} [label = \"{}\"]", name, content.replace('"', "\\\""));
                lines.add_line(node_line);

                let line = format!("{} -> {}", src, name);
                lines.add_line(line);
            }
            Self::Expression(exp) => {
                let content = format!("{:?}", exp);
                let node_line = format!("{} [label = \"{}\"]", name, content.replace('"', "\\\""));
                lines.add_line(node_line);

                let line = format!("{} -> {}", src, name);
                lines.add_line(line);
            }
            Self::Return(val) => {
                let content = format!("return {:?}", val);
                let node_line = format!("{} [label = \"{}\"]", name, content.replace('"', "\\\""));
                lines.add_line(node_line);

                let line = format!("{} -> {}", src, name);
                lines.add_line(line);
            }
            Self::Jump(target) => {
                let content = "Jump".to_string();
                let node_line = format!("{} [label = \"{}\"]", name, content.replace('"', "\\\""));
                lines.add_line(node_line);

                let line = format!("{} -> {}", src, name);
                lines.add_line(line);

                let target_name = target.to_dot(lines, drawn, &Context::new());

                let target_line = format!("{} -> {}", name, target_name);
                lines.add_line(target_line);
            }
            Self::JumpTrue(cond, target) => {
                let content = "JumpTrue".to_string();
                let node_line = format!("{} [label = \"{}\"]", name, content.replace('"', "\\\""));
                lines.add_line(node_line);

                let line = format!("{} -> {}", src, name);
                lines.add_line(line);

                let target_name = target.to_dot(lines, drawn, &Context::new());

                let var_str = format!("{:?}", cond);
                let target_line = format!(
                    "{} -> {} [label = \"{}\"]",
                    name,
                    target_name,
                    var_str.replace('"', "\\\"")
                );
                lines.add_line(target_line);
            }
        };

        name
    }
}
