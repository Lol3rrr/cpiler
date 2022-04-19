use crate::{Optimization, OptimizationPass};

/// This is used to repeat a certain Pass multiple times
pub struct Repeat<P> {
    pass: P,
    count: usize,
}

impl<P> Repeat<P> {
    /// Creates a new Pass that repeats the given Pass a given number of times
    pub fn new(pass: P, repetitions: usize) -> Self {
        Self {
            pass,
            count: repetitions,
        }
    }
}

impl<O> OptimizationPass for Repeat<O>
where
    O: Optimization,
{
    fn name(&self) -> String {
        "Repeat".to_owned()
    }

    fn pass_function(&self, ir: ir::FunctionDefinition) -> ir::FunctionDefinition {
        let mut ir = self.pass.pass_function(ir);

        for _ in 0..self.count {
            ir = self.pass.pass_function(ir);
        }

        ir
    }
}
