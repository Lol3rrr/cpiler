use std::collections::HashMap;

pub struct Groups<R> {
    groups: HashMap<String, R>,
}

impl<R> Groups<R> {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub fn get_group(&self, var: &ir::Variable) -> Option<&R> {
        self.groups.get(&var.name)
    }

    pub fn set_group(&mut self, var: ir::Variable, reg: R) {
        self.groups.insert(var.name, reg);
    }
}
