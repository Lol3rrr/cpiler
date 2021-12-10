use std::sync::{atomic, Arc};

use crate::Type;

/// A single Variable that will only ever be assigned to once
#[derive(Debug, Clone)]
pub struct Variable {
    /// The Name of the Variable
    pub name: String,
    /// The current "Generation" of this Variable Instance
    generation: usize,
    /// The Type of this Variable
    pub ty: Type,
    current_version: Arc<atomic::AtomicUsize>,
}

impl Variable {
    /// Creates a new Variable-Instance with the given Values
    pub fn new<N>(name: N, ty: Type) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            generation: 0,
            ty,
            current_version: Arc::new(atomic::AtomicUsize::new(1)),
        }
    }

    #[cfg(test)]
    pub fn new_test<N>(name: N, generation: usize, ty: Type) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            generation,
            ty,
            current_version: Arc::new(atomic::AtomicUsize::new(generation + 1)),
        }
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    /// Increments the current Generation by one
    pub fn next_gen(&self) -> Self {
        let gen = self.current_version.fetch_add(1, atomic::Ordering::SeqCst);

        Self {
            name: self.name.clone(),
            generation: gen,
            ty: self.ty.clone(),
            current_version: self.current_version.clone(),
        }
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.generation == other.generation && self.ty == other.ty
    }
}
