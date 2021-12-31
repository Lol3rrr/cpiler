use crate::{AScope, AStatement};

pub struct NestedIter<'s> {
    scope: &'s AScope,
    position: usize,
    pending: Vec<AStatement>,
}

impl<'s> NestedIter<'s> {
    pub fn new(scope: &'s AScope) -> Self {
        Self {
            scope,
            position: 0,
            pending: Vec::new(),
        }
    }
}

impl<'s> Iterator for NestedIter<'s> {
    type Item = AStatement;

    fn next(&mut self) -> Option<Self::Item> {
        let temp = match self.pending.pop() {
            Some(p) => p,
            None => {
                let t = self.scope.statements.get(self.position)?;
                self.position += 1;
                t.clone()
            }
        };

        match &temp {
            AStatement::If { body, .. } => {
                let iter = body.statements.iter().rev().cloned();
                self.pending.extend(iter);
            }
            AStatement::WhileLoop { body, .. } => {
                let iter = body.statements.iter().rev().cloned();
                self.pending.extend(iter);
            }
            AStatement::ForLoop { body, .. } => {
                let iter = body.statements.iter().rev().cloned();
                self.pending.extend(iter);
            }
            AStatement::SubScope { inner } => {
                let iter = inner.statements.iter().rev().cloned();
                self.pending.extend(iter);
            }
            _ => {}
        };

        Some(temp)
    }
}
