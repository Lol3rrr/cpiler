#![warn(missing_docs)]
//! The Optimizier works on optimizing the IR for a given Program

use std::collections::HashMap;

use ir::Program;

pub mod optimizations;

mod config;
pub use config::Config;

/// The general Interface for an Optimization
pub trait Optimization
where
    Self: Sized + OptimizationPass,
{
    /// Creates a new Chain Pass with the current one as the first Pass and the given other Pass as
    /// the second one
    fn chain<O>(self, other: O) -> optimizations::Chain<Self, O>
    where
        O: Optimization + Sized,
    {
        optimizations::Chain::new(self, other)
    }

    /// Creates a new Repeat Pass with current one as the Pass and the given number of repetitions
    fn repeat(self, repetitions: usize) -> optimizations::Repeat<Self> {
        optimizations::Repeat::new(self, repetitions)
    }
}

/// The underlying Trait needed for an Optimization Pass
pub trait OptimizationPass {
    /// The Name of the Optimization
    fn name(&self) -> String;

    /// Actually performs an optimization pass on the given IR
    fn pass_function(&self, ir: ir::FunctionDefinition) -> ir::FunctionDefinition;
}

impl<T> Optimization for T where T: OptimizationPass {}

/// This is used to optimize a single Function with the given Configuration
pub fn optimize_func(func: ir::FunctionDefinition, config: &Config) -> ir::FunctionDefinition {
    let mut result = func;
    for pass in config.passes.iter() {
        result = pass.pass_function(result);
    }

    result
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
