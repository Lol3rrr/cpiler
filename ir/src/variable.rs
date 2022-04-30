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
        var: Box<String>,
    },
    /// This marks a Variable as a Temporary Variable that should not be in the final IR
    Temporary,
}

/// The Group of Variables
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct VariableGroup {
    /// The shared Name
    name: Arc<String>,
}

impl VariableGroup {
    /// Checks if the given Variable is a Part of the Variable Group
    pub fn contains<V>(&self, var: V) -> bool
    where
        V: AsRef<Variable>,
    {
        var.as_ref().name == self.name
    }
}

impl From<Variable> for VariableGroup {
    fn from(other: Variable) -> Self {
        Self { name: other.name }
    }
}
impl From<&Variable> for VariableGroup {
    fn from(other: &Variable) -> Self {
        Self {
            name: other.name.clone(),
        }
    }
}

/// A single Variable that will only ever be assigned to once
#[derive(Debug, Clone)]
pub struct Variable {
    /// The Name of the Variable
    name: Arc<String>,
    /// The current "Generation" of this Variable Instance
    generation: usize,
    /// The Type of this Variable
    pub ty: Type,
    /// Extra Metadata for this Variable
    meta: VariableMetadata,
    /// Whether or not the Variable is actually a Global-Variable
    global: bool,
    /// Contains a small Description with the Purpose of this Variable, this will not be used in
    /// comparisons and is mostly used for debuging to help identify which variable is responsible
    /// for what
    description: Option<String>,
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
            name: Arc::new(name.into()),
            generation: 0,
            ty,
            meta,
            global: false,
            description: None,
            current_version: Arc::new(atomic::AtomicUsize::new(1)),
        }
    }

    /// The Name of the Variable
    pub fn name(&self) -> &str {
        self.name.as_ref()
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
    #[must_use]
    pub fn set_meta(mut self, meta: VariableMetadata) -> Self {
        self.meta = meta;
        self
    }
    /// Gets the Metadata for a Variable
    pub fn meta(&self) -> &VariableMetadata {
        &self.meta
    }

    /// This updates the Global-"State" for the Variable
    #[must_use]
    pub fn set_global(mut self, global: bool) -> Self {
        self.global = global;
        self
    }
    /// Whether or not the Variable is a Global
    pub fn global(&self) -> bool {
        self.global
    }

    /// Updates the Description for this Variable
    #[must_use]
    pub fn set_description<D>(mut self, desc: D) -> Self
    where
        D: Into<String>,
    {
        self.description = Some(desc.into());
        self
    }

    /// Returns the Generation of the Variable
    pub fn generation(&self) -> usize {
        self.generation
    }

    /// Increments the current Generation by one
    #[must_use]
    pub fn next_gen(&self) -> Self {
        let gen = self.current_version.fetch_add(1, atomic::Ordering::SeqCst);

        Self {
            name: self.name.clone(),
            generation: gen,
            ty: self.ty.clone(),
            meta: self.meta.clone(),
            global: self.global,
            description: None,
            current_version: self.current_version.clone(),
        }
    }

    /// Checks if a Variable is a Temp Variable
    pub fn is_tmp(&self) -> bool {
        self.name.starts_with("__t_")
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

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.name < other.name {
            return Some(std::cmp::Ordering::Less);
        }
        if self.name > other.name {
            return Some(std::cmp::Ordering::Greater);
        }

        self.generation.partial_cmp(&other.generation)
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Hash for Variable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        state.write_usize(self.generation);
        self.ty.hash(state);
        self.meta.hash(state);
    }
}

impl AsRef<Variable> for Variable {
    fn as_ref(&self) -> &Variable {
        self
    }
}
