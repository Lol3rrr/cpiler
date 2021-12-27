use crate::{Optimization, OptimizationPass};

/// This is used to chain to OptimizationPasses together, if you then apply this Optimization Chain
/// you automatically first apply the given first Pass and then the second Pass
pub struct Chain<F, S> {
    first: F,
    second: S,
}

impl<F, S> Chain<F, S> {
    /// Creates a new Chain Pass that combines the two given Passes
    pub fn new(first: F, second: S) -> Self {
        Self { first, second }
    }
}

impl<F, S> OptimizationPass for Chain<F, S>
where
    F: Optimization,
    S: Optimization,
{
    fn name(&self) -> String {
        "Chain".to_string()
    }

    fn pass_function(&self, ir: ir::FunctionDefinition) -> ir::FunctionDefinition {
        let first_pass = self.first.pass_function(ir);
        self.second.pass_function(first_pass)
    }
}
