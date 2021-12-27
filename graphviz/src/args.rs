use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Args {
    inner: BTreeMap<String, String>,
}

impl Args {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn add<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.inner.insert(key.into(), value.into());
    }

    pub fn line_string(&self) -> String {
        if self.inner.is_empty() {
            return "".to_string();
        }

        let mut result = String::new();
        result.push('[');
        for (arg_name, arg_value) in self.inner.iter() {
            result.push_str(arg_name);
            result.push_str("=\"");
            result.push_str(arg_value);
            result.push_str("\" ");
        }
        result.push(']');
        result
    }
}
