use std::collections::HashMap;

use crate::general;

use super::{BlockIndex, PhiEntry};

/// The Value in the simpler IR
pub type Value = general::Value<BlockIndex>;

impl Value {
    pub(crate) fn from_complex(
        src: crate::Value,
        block_map: &HashMap<*const crate::InnerBlock, BlockIndex>,
    ) -> Self {
        match src {
            crate::Value::Unknown => Self::Unknown,
            crate::Value::Constant(con) => Self::Constant(con),
            crate::Value::Variable(var) => Self::Variable(var),
            crate::Value::Expression(exp) => Self::Expression(exp),
            crate::Value::Phi { sources } => {
                let n_sources: Vec<_> = sources
                    .into_iter()
                    .filter_map(|s| {
                        let b = block_map.get(&s.block.as_ptr())?.clone();
                        Some(PhiEntry {
                            var: s.var,
                            block: b,
                        })
                    })
                    .collect();

                Self::Phi { sources: n_sources }
            }
        }
    }
}
