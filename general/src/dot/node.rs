use super::args::Args;

#[derive(Debug)]
pub struct Node {
    pub(crate) name: String,
    pub(crate) args: Args,
}

impl Node {
    pub fn new<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            args: Args::new(),
        }
    }

    pub fn add_label<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        self.args.add(name, value);
        self
    }

    pub(crate) fn line(&self) -> String {
        let args = self.args.line_string();
        format!("{} {};\n", self.name, args)
    }
}
