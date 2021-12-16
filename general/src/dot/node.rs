use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Node {
    pub(crate) name: String,
    pub(crate) args: BTreeMap<String, String>,
}

impl Node {
    pub fn new<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
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
        let args = if self.args.is_empty() {
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

        format!("{} {};\n", self.name, args)
    }
}
