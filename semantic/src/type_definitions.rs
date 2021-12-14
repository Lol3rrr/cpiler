use std::collections::HashMap;

use syntax::Identifier;

use crate::AType;

/// This is used to map TypeDef definitions or Struct Names to their respective underlying Types
#[derive(Debug, PartialEq, Clone)]
pub struct TypeDefinitions {
    defined: HashMap<String, AType>,
}

impl TypeDefinitions {
    /// Creates a new Empty Type Definitions instance
    pub fn new() -> Self {
        Self {
            defined: HashMap::new(),
        }
    }

    /// Creates a new independant Instance based on the given other Instance, this is especially
    /// useful for parsing sub-categories or scopes as you still get access to the outer Data
    /// but can also add new local Types to it without modifying the outer Structure as well
    pub fn based(other: &Self) -> Self {
        Self {
            defined: other.defined.clone(),
        }
    }

    /// Adds a new Mapping for the given Identifier to the Target Type
    pub fn add_definition(&mut self, ident: Identifier, target: AType) {
        self.defined.insert(ident.0.data, target);
    }

    /// Loads the Type that belongs to the given Identifier, if any exists
    pub fn get_definition(&self, ident: &Identifier) -> Option<&AType> {
        self.defined.get(&ident.0.data)
    }
}

impl Default for TypeDefinitions {
    fn default() -> Self {
        Self::new()
    }
}
