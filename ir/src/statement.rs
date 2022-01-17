use std::fmt::Debug;

use crate::{
    comp::CompareGraph,
    dot::{Context, DrawnBlocks},
    general, BasicBlock, ToDot, WeakBlockPtr,
};

/// A Statement in the IR contains a single "Instruction", like evaluating an expression and/or
/// storing its result in a new Variable or jumping to a different Point in the Program
pub type Statement = general::Statement<BasicBlock, WeakBlockPtr>;

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

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.print(f, |b| format!("0x{:x}", b.as_ptr() as usize))
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
            Self::SaveGlobalVariable { name: var_name } => {
                let content = format!("SaveGlobalVariable {:?}", var_name);
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
            Self::InlineAsm {
                template,
                inputs,
                output,
            } => {
                let content = format!("InlineASM {} with {:?} into {:?}", template, inputs, output);
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
