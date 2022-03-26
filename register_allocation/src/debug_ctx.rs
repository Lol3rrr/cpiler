pub struct DebugContext {
    states: Vec<Step>,
}

#[derive(Debug)]
pub enum Step {
    State(String),
    Spill(ir::Variable),
}

impl DebugContext {
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }

    pub fn add_state(&mut self, state: &ir::FunctionDefinition) {
        self.states
            .push(Step::State(ir::text_rep::generate_text_rep(state)));
    }

    pub fn add_var_spill(&mut self, var: ir::Variable) {
        self.states.push(Step::Spill(var));
    }

    pub fn get_steps(&self) -> impl Iterator<Item = &Step> + '_ {
        self.states.iter()
    }
}
