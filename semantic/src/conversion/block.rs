use ir::{Statement, Variable};

#[derive(Debug)]
pub struct BuildBlock {
    statements: Vec<Statement>,
}

impl BuildBlock {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    /// Adds a new Statement to the statements for this given Block that is build
    pub fn add_statement(&mut self, statment: Statement) {
        self.statements.push(statment);
    }

    /// This is used to get the latest Variable for the given Name
    pub fn get_definition(&self, name: &str) -> Option<&Variable> {
        for tmp in self.statements.iter().rev() {
            dbg!(&tmp);
        }

        todo!("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_local_definition() {}
}
