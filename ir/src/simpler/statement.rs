use std::{collections::HashMap, fmt::Debug};

use crate::general;

use super::{BlockIndex, Value};

pub type Statement = general::Statement<BlockIndex, BlockIndex>;

impl Statement {
    pub(crate) fn from_complex(
        src: crate::Statement,
        block_map: &HashMap<*const crate::InnerBlock, BlockIndex>,
    ) -> Self {
        match src {
            crate::Statement::Assignment { target, value } => {
                let n_value = Value::from_complex(value, block_map);

                Self::Assignment {
                    target,
                    value: n_value,
                }
            }
            crate::Statement::WriteMemory { target, value } => {
                let n_value = Value::from_complex(value, block_map);

                Self::WriteMemory {
                    target,
                    value: n_value,
                }
            }
            crate::Statement::SaveVariable { var } => Self::SaveVariable { var },
            crate::Statement::InlineAsm {
                template,
                inputs,
                output,
            } => Self::InlineAsm {
                template,
                inputs,
                output,
            },
            crate::Statement::Call { name, arguments } => Self::Call { name, arguments },
            crate::Statement::Jump(target) => {
                let n_target = block_map.get(&target.as_ptr()).unwrap().clone();
                Self::Jump(n_target)
            }
            crate::Statement::JumpTrue(var, target) => {
                let n_target = block_map.get(&target.as_ptr()).unwrap().clone();

                Self::JumpTrue(var, n_target)
            }
            crate::Statement::Return(var) => Self::Return(var),
        }
    }
}

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assignment { target, value } => f
                .debug_struct("Assignment")
                .field("target", target)
                .field("value", value)
                .finish(),
            Self::WriteMemory { target, value } => f
                .debug_struct("WriteMemory")
                .field("target", target)
                .field("value", value)
                .finish(),
            Self::SaveVariable { var } => f.debug_struct("SaveVariable").field("var", var).finish(),
            Self::InlineAsm {
                template,
                inputs,
                output,
            } => f
                .debug_struct("InlineAsm")
                .field("template", template)
                .field("inputs", inputs)
                .field("output", output)
                .finish(),
            Self::Call { name, arguments } => f
                .debug_struct("Call")
                .field("name", name)
                .field("arguments", arguments)
                .finish(),
            Self::Jump(target) => f.debug_tuple("Jump").field(target).finish(),
            Self::JumpTrue(var, target) => {
                f.debug_tuple("JumpTrue").field(var).field(target).finish()
            }
            Self::Return(var) => f.debug_tuple("Return").field(var).finish(),
        }
    }
}
