use std::fmt::Debug;

use crate::{
    comp::CompareGraph,
    dot::{Context, DrawnBlocks},
    BasicBlock, Operand, ToDot, Value, Variable,
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
    /// This writes the Value to some location in memory, mostly done through a Pointer
    WriteMemory {
        /// The Target on where to write the Value
        target: Operand,
        /// The Value
        value: Value,
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
    /// This indicates that the Variable should be reloaded, usually from the Stack
    LoadVariable {
        /// The Variable that should be loaded
        var: Variable,
    },
    /// This indicates that the Variable should be saved to a Register and frees up the current
    /// Register to be freed up again
    UnloadVariable {
        /// The Variable to unload
        var: Variable,
    },
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
            (Self::LoadVariable { var: s_var }, Self::LoadVariable { var: o_var }) => {
                s_var == o_var
            }
            (Self::UnloadVariable { var: s_var }, Self::UnloadVariable { var: o_var }) => {
                s_var == o_var
            }
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
            Self::Assignment { target, value } => f
                .debug_struct("Assignment")
                .field("target", &target)
                .field("value", &value)
                .finish(),
            Self::SaveVariable { var } => {
                f.debug_struct("SaveVariable").field("var", &var).finish()
            }
            Self::LoadVariable { var } => {
                f.debug_struct("LoadVariable").field("var", &var).finish()
            }
            Self::UnloadVariable { var } => {
                f.debug_struct("UnloadVariable").field("var", &var).finish()
            }
            Self::WriteMemory { target, value } => f
                .debug_struct("WriteMemory")
                .field("target", &target)
                .field("value", &value)
                .finish(),
            Self::Call { name, arguments } => {
                write!(f, "Call {:?} with {:?}", name, arguments)
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
    fn to_dot(
        &self,
        lines: &mut dyn graphviz::Graph,
        drawn: &mut DrawnBlocks,
        ctx: &Context,
    ) -> String {
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

        if drawn.contains(&name) {
            return name;
        }
        drawn.add_block(&name);

        match self {
            Self::Assignment { target, value } => {
                let content = format!("{:?} = {:?}", target, value);
                lines.add_node(
                    graphviz::Node::new(&name).add_label("label", content.replace('"', "\\\"")),
                );

                lines.add_edge(graphviz::Edge::new(src, &name));
            }
            Self::SaveVariable { var } => {
                let content = format!("SaveVariable {:?}", var);
                lines.add_node(
                    graphviz::Node::new(&name).add_label("label", content.replace('"', "\\\"")),
                );

                lines.add_edge(graphviz::Edge::new(src, &name));
            }
            Self::LoadVariable { var } => {
                let content = format!("LoadVariable {:?}", var);
                lines.add_node(
                    graphviz::Node::new(&name).add_label("label", content.replace('"', "\\\"")),
                );

                lines.add_edge(graphviz::Edge::new(src, &name));
            }
            Self::UnloadVariable { var } => {
                let content = format!("UnloadVariable {:?}", var);
                lines.add_node(
                    graphviz::Node::new(&name).add_label("label", content.replace('"', "\\\"")),
                );

                lines.add_edge(graphviz::Edge::new(src, &name));
            }
            Self::WriteMemory { target, value } => {
                let content = format!("WriteMemory {:?} = {:?}", target, value);
                lines.add_node(
                    graphviz::Node::new(&name).add_label("label", content.replace('"', "\\\"")),
                );

                lines.add_edge(graphviz::Edge::new(src, &name));
            }
            Self::Call { .. } => {
                let content = format!("{:?}", self);
                lines.add_node(
                    graphviz::Node::new(&name).add_label("label", content.replace('"', "\\\"")),
                );

                lines.add_edge(graphviz::Edge::new(src, &name));
            }
            Self::Return(val) => {
                let content = format!("return {:?}", val);
                lines.add_node(
                    graphviz::Node::new(&name).add_label("label", content.replace('"', "\\\"")),
                );

                lines.add_edge(graphviz::Edge::new(src, &name));
            }
            Self::Jump(target) => {
                let content = "Jump".to_string();
                lines.add_node(
                    graphviz::Node::new(&name).add_label("label", content.replace('"', "\\\"")),
                );

                lines.add_edge(graphviz::Edge::new(src, &name));

                let target_name = target.name(ctx);

                lines.add_edge(graphviz::Edge::new(&name, target_name));
            }
            Self::JumpTrue(cond, target) => {
                let content = "JumpTrue".to_string();
                lines.add_node(
                    graphviz::Node::new(&name).add_label("label", content.replace('"', "\\\"")),
                );

                lines.add_edge(graphviz::Edge::new(src, &name));

                let target_name = target.name(ctx);

                let var_str = format!("{:?}", cond);
                lines.add_edge(
                    graphviz::Edge::new(&name, target_name)
                        .add_label("label", var_str.replace('"', "\\\"")),
                );
            }
        };

        name
    }

    fn name(&self, ctx: &Context) -> String {
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

        format!("block_{}_s{}", block_ptr, number_in_block)
    }
}

impl Statement {
    /// Returns a list of all the used Variables in this Statement
    ///
    /// # Note
    /// This does not contain the Targets of Assignment Statements
    pub fn used_vars(&self) -> Vec<Variable> {
        match self {
            Self::Assignment { value, .. } => value.used_vars(),
            Self::SaveVariable { var } => vec![var.clone()],
            Self::LoadVariable { .. } => Vec::new(),
            Self::UnloadVariable { var } => vec![var.clone()],
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
            Self::Return(None) => Vec::new(),
            Self::Return(Some(var)) => vec![var.clone()],
            Self::Jump(_) => Vec::new(),
            Self::JumpTrue(var, _) => vec![var.clone()],
        }
    }
}
