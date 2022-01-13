use super::args::Args;

/// A Single Node in a Graph
#[derive(Debug)]
pub struct Node {
    pub(crate) name: String,
    pub(crate) args: Args,
}

impl Node {
    /// Creates a new Node with the given Name
    ///
    /// # Note
    /// The Name of a Node should be unique in a Graph
    pub fn new<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            args: Args::new(),
        }
    }

    /// Adds a label to the Node itself, which allows for different Styles per Node or overwriting
    /// the Content of the Node itself
    #[must_use]
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
