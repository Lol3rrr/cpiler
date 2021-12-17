#![warn(missing_docs)]
//! The Optimizier works on optimizing the IR for a given Program

use std::collections::HashMap;

use ir::Program;

pub mod optimizations;

mod config;
pub use config::Config;

/// The general Interface for a given Optimization
pub trait Optimization {
    /// The Name of the Optimization
    fn name(&self) -> String;

    /// Actually performs an optimization pass on the given IR
    fn pass_function(&self, ir: ir::FunctionDefinition) -> ir::FunctionDefinition;
}

/// This will actually apply the given Optimization Config to the Program
pub fn optimize(ir: Program, config: Config) -> Program {
    let mut result = Program {
        global: ir.global,
        functions: HashMap::new(),
    };

    for (name, def) in ir.functions.into_iter() {
        let mut def = def;
        for pass in &config.passes {
            def = pass.pass_function(def);
        }

        result.functions.insert(name, def);
    }

    result
}
