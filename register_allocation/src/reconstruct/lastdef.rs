use std::collections::HashMap;

use ir::{Variable, VariableGroup, WeakBlockPtr};

#[derive(Debug, PartialEq, Clone)]
pub enum LastDef {
    Single(Variable, WeakBlockPtr),
    Two((Variable, WeakBlockPtr), (Variable, WeakBlockPtr)),
}

#[derive(Debug, PartialEq, Clone)]
pub struct LastDefinition {
    inner: HashMap<VariableGroup, LastDef>,
}

impl LastDefinition {
    pub fn new() -> LastDefinition {
        LastDefinition {
            inner: HashMap::new(),
        }
    }

    pub fn defined_single(&mut self, var: Variable, block: WeakBlockPtr) {
        let group: VariableGroup = var.clone().into();
        self.inner.insert(group, LastDef::Single(var, block));
    }
    pub fn defined_two(&mut self, var1: (Variable, WeakBlockPtr), var2: (Variable, WeakBlockPtr)) {
        let group: VariableGroup = var1.0.clone().into();
        self.inner.insert(group, LastDef::Two(var1, var2));
    }

    pub fn get_last(&self, var: &Variable) -> Option<&LastDef> {
        let group: VariableGroup = var.clone().into();
        self.inner.get(&group)
    }

    pub fn retain<F>(&mut self, func: F)
    where
        F: Fn(&VariableGroup, &mut LastDef) -> bool,
    {
        self.inner.retain(func)
    }

    pub fn contains<G>(&self, group: G) -> bool
    where
        G: Into<VariableGroup>,
    {
        let group: VariableGroup = group.into();
        self.inner.contains_key(&group)
    }

    pub fn groups(&self) -> impl Iterator<Item = &VariableGroup> {
        self.inner.keys()
    }

    pub fn intersection(&self, other: &Self) -> Self {
        let left_same = self
            .inner
            .iter()
            .filter(|(k, v)| other.inner.get(k) == Some(v));
        let right_same = other
            .inner
            .iter()
            .filter(|(k, v)| self.inner.get(k) == Some(v));
        let joined = left_same.chain(right_same);
        let junc_map: HashMap<_, _> = joined.map(|(k, v)| (k.clone(), v.clone())).collect();

        Self { inner: junc_map }
    }

    pub fn joined_iter<'a, 'o, 'i>(
        &'a self,
        other: &'o Self,
    ) -> impl Iterator<Item = (&VariableGroup, (&LastDef, &LastDef))> + 'i
    where
        'a: 'i,
        'o: 'a,
        'o: 'i,
    {
        self.inner
            .iter()
            .map(|(k, v)| (k, (v, other.inner.get(k).unwrap())))
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}
