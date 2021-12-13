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

impl From<Lines> for Vec<String> {
    fn from(src: Lines) -> Self {
        src.inner
    }
}

impl Default for Lines {
    fn default() -> Self {
        Self::new()
    }
}
