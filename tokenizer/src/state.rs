use general::{Span, SpanRef};

pub struct TokenizeState {
    start: usize,
    env: Environment,
}

/// The current Context in which the Text is being parsed, like normal Code, Comments, etc.
pub enum Environment {
    /// The Normal Code context
    Code,
    /// A Single Line Comment
    SLComment,
    /// A Multi Line Comment
    MLComment,
    /// The Compiler Directive Context
    CompilerDirective,
    /// The String Literal Context
    StringLiteral,
}

impl TokenizeState {
    pub fn new() -> Self {
        Self {
            start: 0,
            env: Environment::Code,
        }
    }

    pub fn env(&self) -> &Environment {
        &self.env
    }

    pub fn current_sub<'a>(&self, root: &'a Span, end: usize) -> Option<SpanRef<'a>> {
        let range = self.start..end;
        if end < self.start {
            return None;
        }

        root.sub_span(range)
    }

    pub fn move_start(&mut self, new: usize) {
        self.start = new;
    }

    pub fn switch_env(&mut self, new: Environment) {
        self.env = new;
    }
}
