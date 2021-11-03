pub mod files {

    use general::Span;

    use crate::Loader;

    pub struct FileLoader {}

    impl FileLoader {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Loader for FileLoader {
        type LoadError = std::io::Error;

        fn load_file(&self, path: &str) -> Result<general::Span, Self::LoadError> {
            let raw_content = std::fs::read(path)?;
            let content = String::from_utf8(raw_content).unwrap();

            let res_span = Span::new_source(path, content);

            Ok(res_span)
        }
    }
}
