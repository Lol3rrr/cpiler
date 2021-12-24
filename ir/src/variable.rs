use std::{
    hash::Hash,
    sync::{atomic, Arc},
};

use crate::Type;

/// Contains extra Metadata, for different kinds of Variables
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum VariableMetadata {
    /// This is used to indicate that the Variable has no extra Metadata assosicated with it
    Empty,
    /// The Metadata for a Pointer to some Address
    Pointer,
    /// The Metadata for a Pointer to another Variable
    VarPointer {
        /// The Variable that this Variable Points to
        var: Box<Variable>,
    },
}

/// A single Variable that will only ever be assigned to once
#[derive(Debug, Clone)]
pub struct Variable {
    /// The Name of the Variable
    pub name: String,
    /// The current "Generation" of this Variable Instance
    generation: usize,
    /// The Type of this Variable
    pub ty: Type,
    /// Extra Metadata for this Variable
    meta: VariableMetadata,
    current_version: Arc<atomic::AtomicUsize>,
}

impl Variable {
    /// Creates a new Variable-Instance with the given Values, this function will infer the Basic
    /// Metadata for the Variable
    pub fn new<N>(name: N, ty: Type) -> Self
    where
        N: Into<String>,
    {
        let meta = match &ty {
            Type::Pointer(_) => VariableMetadata::Pointer,
            _ => VariableMetadata::Empty,
        };

        Self {
            name: name.into(),
            generation: 0,
            ty,
            meta,
            current_version: Arc::new(atomic::AtomicUsize::new(1)),
        }
    }

    /// Creates a new Temporary Variable with the given number
    pub fn tmp(number: usize, ty: Type) -> Self {
        Self::new(format!("__t_{}", number), ty)
    }

    #[cfg(test)]
    pub fn new_test<N>(name: N, generation: usize, ty: Type) -> Self
    where
        N: Into<String>,
    {
        let mut tmp = Self::new(name, ty);
        tmp.generation = generation;
        tmp
    }

    /// This updates the Metadata for the Variable
    pub fn set_meta(mut self, meta: VariableMetadata) -> Self {
        self.meta = meta;
        self
    }
    /// Gets the Metadata for a Variable
    pub fn meta(&self) -> &VariableMetadata {
        &self.meta
    }

    /// Returns the Generation of the Variable
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
            meta: self.meta.clone(),
            current_version: self.current_version.clone(),
        }
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.generation == other.generation
            && self.ty == other.ty
            && self.meta == other.meta
    }
}

impl Eq for Variable {}

impl Hash for Variable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        state.write_usize(self.generation);
        self.ty.hash(state);
        self.meta.hash(state);
    }
}
