use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Edge {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) args: BTreeMap<String, String>,
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
            args: BTreeMap::new(),
        }
    }

    pub fn add_label<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        self.args.insert(name.into(), value.into());
        self
    }

    pub(crate) fn line(&self) -> String {
        let label = if self.args.is_empty() {
            String::new()
        } else {
            let mut result = String::new();
            result.push('[');
            for (arg_name, arg_value) in self.args.iter() {
                result.push_str(arg_name);
                result.push_str("=\"");
                result.push_str(arg_value);
                result.push_str("\" ");
            }
            result.push(']');
            result
        };

        format!("{} -> {} {};\n", self.from, self.to, label)
    }
}
