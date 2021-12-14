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
//!
//! # References
//! These are a couple of Resources that inspired this IR design or include potential improvements
//! to the IR in Future iterations:
//! * [SSA Construction](https://pp.info.uni-karlsruhe.de/uploads/publikationen/braun13cc.pdf)
//! * [Array SSA Form](https://www.cs.purdue.edu/homes/suresh/590s-Fall2002/papers/ArraySSApopl98.pdf)
//! * [Extended SSA: Pointers etc.](https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.17.1802&rep=rep1&type=pdf)

use std::{collections::HashMap, fmt::Debug};

mod variable;
use dot::{Context, DrawnBlocks};
pub use variable::*;

mod dot;
pub use dot::ToDot;

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

mod comp;

/// The overall Program Structure that contains all the needed information about the Program itself
#[derive(Clone, PartialEq)]
pub struct Program {
    /// This contains definitions for Global Variables that need to be usable by the function
    /// definitions
    pub global: BasicBlock,
    /// The various Function Definitions in the Program
    pub functions: HashMap<String, FunctionDefinition>,
}

impl Program {
    /// Generates the needed Dot Graphviz Representation to allow for easier visualization of the
    /// Program
    pub fn to_dot(&self) -> String {
        let mut graph = general::dot::Graph::new();
        let mut drawn = DrawnBlocks::new();

        for (_, func_def) in self.functions.iter() {
            func_def.to_dot(&mut graph, &mut drawn, &Context::new());
        }

        graph.finalize()
    }
}

impl Debug for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f_struct = f.debug_struct("Program");

        f_struct.field("global", &format!("{:?}", self.global));
        f_struct.field("functions", &self.functions);

        Ok(())
    }
}
