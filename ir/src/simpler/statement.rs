use std::{collections::HashMap, fmt::Debug};

use crate::general;

use super::{BlockIndex, Value};

/// The Statement in the simpler IR
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
            crate::Statement::WriteMemory { target, value } => Self::WriteMemory { target, value },
            crate::Statement::SaveVariable { var } => Self::SaveVariable { var },
            crate::Statement::SaveGlobalVariable { name, value } => {
                Self::SaveGlobalVariable { name, value }
            }
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
            crate::Statement::Jump(target, meta) => {
                let n_target = block_map.get(&target.as_ptr()).unwrap().clone();
                Self::Jump(n_target, meta)
            }
            crate::Statement::JumpTrue(var, target, meta) => {
                let n_target = block_map.get(&target.as_ptr()).unwrap().clone();

                Self::JumpTrue(var, n_target, meta)
            }
            crate::Statement::Return(var) => Self::Return(var),
        }
    }
}

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.print(f, |b| format!("{:?}", b))
    }
}
