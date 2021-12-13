pub struct Lines {
    inner: Vec<String>,
}

impl Lines {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn add_line<L>(&mut self, line: L)
    where
        L: Into<String>,
    {
        let line = line.into();
        self.inner.push(line);
    }
}

impl Into<Vec<String>> for Lines {
    fn into(self) -> Vec<String> {
        self.inner
    }
}
