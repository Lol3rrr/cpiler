//! This module contains a simplified Structure to Represent a Function in a Program.
//!
//! Limitations:
//! Currently this does not support a Global Block

use std::collections::HashMap;

use crate::{general, Type};

mod block;
pub use block::*;

mod value;
pub use value::*;

mod statement;
pub use statement::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BlockIndex(usize);

pub type PhiEntry = general::PhiEntry<BlockIndex>;

pub struct Function {
    name: String,
    arguments: Vec<(String, Type)>,
    return_ty: Type,
    start: usize,
    blocks: Vec<Block>,
}

impl From<crate::FunctionDefinition> for Function {
    fn from(src: crate::FunctionDefinition) -> Self {
        let mut blocks: Vec<Block> = Vec::new();
        let mut block_mapping: HashMap<*const crate::InnerBlock, BlockIndex> = HashMap::new();

        let src_preds = src.block.get_predecessors();
        if !src_preds.is_empty() {
            let global = src_preds.get(0).unwrap().upgrade().unwrap();
            let mut n_global = Block {
                predecessors: Vec::new(),
                statments: Vec::new(),
            };
            n_global.convert_from_complex(global, &block_mapping);
            dbg!(&n_global);

            todo!()
        }

        for raw in src.block.block_iter() {
            let ptr = raw.as_ptr();

            let index = blocks.len();
            blocks.push(Block {
                predecessors: Vec::new(),
                statments: Vec::new(),
            });

            block_mapping.insert(ptr, BlockIndex(index));
        }

        for convert_block in src.block.block_iter() {
            let target_index = block_mapping.get(&convert_block.as_ptr()).cloned().unwrap();
            let target_block = blocks.get_mut(target_index.0).unwrap();

            target_block.convert_from_complex(convert_block, &block_mapping);
        }

        let start_index = block_mapping.get(&src.block.as_ptr()).cloned().unwrap();

        Self {
            name: src.name,
            arguments: src.arguments,
            return_ty: src.return_ty,
            start: start_index.0,
            blocks,
        }
    }
}
