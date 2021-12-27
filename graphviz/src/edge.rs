use super::args::Args;

/// A single Edge in the Graph
#[derive(Debug)]
pub struct Edge {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) args: Args,
}

impl Edge {
    /// Creates a new Edge between the src and dest
    pub fn new<S, D>(src: S, dest: D) -> Self
    where
        S: Into<String>,
        D: Into<String>,
    {
        Self {
            from: src.into(),
            to: dest.into(),
            args: Args::new(),
        }
    }

    /// Adds the given Label to the Edge, this allows for things like changing the Style of it or
    /// forcing a certain layout
    pub fn add_label<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        self.args.add(name, value);
        self
    }

    pub(crate) fn line(&self) -> String {
        let labels = self.args.line_string();

        format!("{} -> {} {};\n", self.from, self.to, labels)
    }
}
