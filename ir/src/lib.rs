#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use std::{collections::HashMap, fmt::Debug};

mod variable;
use dot::{Context, DrawnBlocks};
use graphviz::Graph;
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

mod interference;
pub use interference::*;

mod dominance;
pub use dominance::*;

mod comp;

pub mod simpler;

mod general;
pub use crate::general::JumpMetadata;

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
        let mut graph = graphviz::RootGraph::new();
        let mut drawn = DrawnBlocks::new();

        let ctx = Context::new();

        let mut global_graph = graphviz::SubGraph::new("global")
            .cluster()
            .arg("label", "Global");
        self.global.to_dot(&mut global_graph, &mut drawn, &ctx);
        graph.add_subgraph(global_graph);

        for (_, func_def) in self.functions.iter() {
            func_def.to_dot(&mut graph, &mut drawn, &ctx);
        }

        graph.finalize()
    }
}

impl Debug for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f_struct = f.debug_struct("Program");

        f_struct.field("global", &self.global);
        f_struct.field("functions", &self.functions);

        Ok(())
    }
}
