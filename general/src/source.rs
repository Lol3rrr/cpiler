use std::{fmt::Debug, ops::Range};

#[derive(PartialEq, Clone)]
pub struct Source {
    name: String,
    content: String,
}

impl Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source {{ name = {} }}", self.name)
    }
}

impl Source {
    pub fn new<N, C>(name: N, content: C) -> Self
    where
        N: Into<String>,
        C: Into<String>,
    {
        Self {
            name: name.into(),
            content: content.into(),
        }
    }

    pub fn get(&self, range: Range<usize>) -> Option<&str> {
        self.content.get(range)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn sub_content(&self, range: Range<usize>) -> Option<&str> {
        self.content.get(range)
    }

    pub fn get_lines_of_chars(&self, range: Range<usize>) -> Option<(usize, &str)> {
        let mut last_line_beginning = 0;
        for (i, c) in self.content.char_indices() {
            if i >= range.start {
                break;
            }
            match c {
                '\n' => {
                    last_line_beginning = i + 1;
                }
                _ => {}
            };
        }

        let mut last_line_end = range.end;
        let mut update_line = false;
        for (i, c) in self.content[range.end..].char_indices() {
            match c {
                '\n' => {
                    last_line_end = i + range.end;
                    update_line = true;
                    break;
                }
                _ => {}
            };
        }
        if !update_line {
            last_line_end = self.content.len();
        }

        let lines = &self.content[last_line_beginning..last_line_end];

        Some((last_line_beginning, lines))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_line_break() {
        let source_content = "Testing";
        let source = Source::new("test", source_content);

        let expected = Some((0, "Testing"));

        let result = source.get_lines_of_chars(1..3);

        assert_eq!(expected, result);
    }

    #[test]
    fn first_line() {
        let source_content = "Testing
other
third";
        let source = Source::new("test", source_content);

        let expected = Some((0, "Testing"));

        let result = source.get_lines_of_chars(1..3);

        assert_eq!(expected, result);
    }

    #[test]
    fn middle_line() {
        let source_content = "Testing
other
third";
        let source = Source::new("test", source_content);

        let expected = Some((8, "other"));

        let result = source.get_lines_of_chars(8..10);

        assert_eq!(expected, result);
    }

    #[test]
    fn multiline() {
        let source_content = "Testing
other
third";
        let source = Source::new("test", source_content);

        let expected = Some((0, "Testing\nother"));

        let result = source.get_lines_of_chars(0..10);

        assert_eq!(expected, result);
    }
}
