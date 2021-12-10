#![warn(missing_docs)]
//! # General
//! This IR is in the SSA-Form and in general is designed to be fairly easy to use and understand.
//! To accomplish this, there are a couple of factors:
//!
//! ## No nested Expression
//! This means that the Operands for a given Expression can only be a Variable or a Constant. This
//! will cause more Statements to be emitted in the IR as all the nested Expressions need to be
//! broken up into smaller pieces and need to be stored in temporary Variables. However this makes
//! the optimizations easier to implement down the Line and also allows for easier Code-Generation
//! in the End because they are already in mostly the right format for it to be translated more or
//! less directly.
//!
//! ## Only Tracking at Scalar-Variable level
//! This means that in cases where we have a Pointer, Array or Struct it treats any modification of
//! the underlying Data or any of its Members is seen as a modification of the Variable itself.
//! This simplifies the overall Structure as we dont need to track any extra Data depending on what
//! type of Variable it is, but also means that we lost some optimization opportunities and also
//! likely produce less efficient code as we have to reread them more often

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

mod variable;
pub use variable::*;

mod ty;
pub use ty::*;

mod value;
pub use value::*;

mod expression;
pub use expression::*;

mod block;
pub use block::*;

mod function;
pub use function::*;

mod statement;
pub use statement::*;

/// The overall Program Structure that contains all the needed information about the Program itself
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// This contains definitions for Global Variables that need to be usable by the function
    /// definitions
    pub global: Arc<BasicBlock>,
    /// The various Function Definitions in the Program
    pub functions: HashMap<String, FunctionDefinition>,
}

impl Program {
    pub fn to_dot(&self) -> String {
        let mut lines = Vec::new();
        let mut drawn = HashSet::new();

        for (func_name, func_def) in self.functions.iter() {
            func_def.to_dot(&func_name, &mut lines, &mut drawn);
        }

        let mut result = "digraph G {\n".to_string();
        for line in lines {
            result.push_str("  ");
            result.push_str(&line);
            result.push_str(";\n");
        }
        result.push_str("}");

        result
    }
}