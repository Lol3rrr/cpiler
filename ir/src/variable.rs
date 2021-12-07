use crate::Type;

/// A single Variable that will only ever be assigned to once
#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    /// The Name of the Variable
    pub name: String,
    /// The current "Generation" of this Variable Instance
    pub generation: usize,
    /// The Type of this Variable
    pub ty: Type,
}

impl Variable {
    /// Creates a new Variable-Instance with the given Values
    pub fn new<N>(name: N, generation: usize, ty: Type) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            generation,
            ty,
        }
    }

    /// Increments the current Generation by one
    pub fn next_generation(&mut self) {
        self.generation += 1;
    }
}
