use crate::Optimization;

/// The Config used for running the optimizier
pub struct Config {
    pub(crate) passes: Vec<Box<dyn Optimization>>,
}

impl Config {
    /// Creates a new empty Config
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Adds a new Optimization-Pass to the Config
    pub fn add_pass<O>(&mut self, pass: O)
    where
        O: Optimization + 'static,
    {
        let pass = Box::new(pass);

        self.passes.push(pass);
    }
}
