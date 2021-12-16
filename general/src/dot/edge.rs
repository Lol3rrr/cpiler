use super::args::Args;

#[derive(Debug)]
pub struct Edge {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) args: Args,
}

impl Edge {
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
