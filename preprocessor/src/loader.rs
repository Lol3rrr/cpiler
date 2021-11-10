use std::path::PathBuf;

use general::Span;

use crate::pir::{into_pir, PIR};

pub struct LoadDirective {
    pub local_root: Option<PathBuf>,
    pub relative_path: PathBuf,
}

pub trait Loader {
    type LoadError: std::error::Error;

    /// Loads the File at the given Path relative to the current Directory/Root
    fn load_file(&self, path: LoadDirective) -> Result<Span, Self::LoadError>;

    fn load_as_pir(&self, path: LoadDirective) -> Result<Vec<PIR>, Self::LoadError> {
        let span = self.load_file(path)?;

        let tokens = tokenizer::tokenize(span);

        let pir = into_pir(tokens);
        Ok(pir.collect())
    }
}

pub mod files {

    use std::path::PathBuf;

    use general::Span;

    use crate::Loader;

    use super::LoadDirective;

    pub struct FileLoader {
        lib_roots: Vec<PathBuf>,
    }

    impl FileLoader {
        pub fn new() -> Self {
            Self {
                lib_roots: Vec::new(),
            }
        }

        pub fn add_lib_root(&mut self, path: PathBuf) {
            self.lib_roots.push(path);
        }
    }

    impl Loader for FileLoader {
        type LoadError = std::io::Error;

        fn load_file(&self, path: LoadDirective) -> Result<general::Span, Self::LoadError> {
            let roots = {
                let initial = match path.local_root {
                    Some(root) => vec![root],
                    None => vec![],
                };

                initial.into_iter().chain(self.lib_roots.iter().cloned())
            };

            for mut root in roots {
                let path = {
                    root.push(&path.relative_path);
                    root
                };

                let raw_content = match std::fs::read(&path) {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let content = String::from_utf8(raw_content).unwrap();

                let path_str = path.to_str().unwrap();
                let res_span = Span::new_source(path_str, content);

                return Ok(res_span);
            }

            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find the File in any of the Roots",
            ))
        }
    }
}
