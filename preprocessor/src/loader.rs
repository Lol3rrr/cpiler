use std::path::PathBuf;

use general::Span;

use crate::{
    pir::{into_pir, PirIterator},
    state::State,
};

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct LoadDirective {
    pub local_root: Option<PathBuf>,
    pub relative_path: PathBuf,
}

pub trait Loader {
    type LoadError: std::error::Error;

    /// Loads the File at the given Path relative to the current Directory/Root
    fn load_file(&self, path: LoadDirective) -> Result<Span, Self::LoadError>;

    fn load_as_pir(
        &self,
        path: LoadDirective,
        state: &mut State,
    ) -> Result<PirIterator<tokenizer::TokenIter>, Self::LoadError> {
        let span = self.load_file(path)?;

        state.add_included_file(span.source().name().to_string());

        let tokens = tokenizer::tokenize(span);

        let pir = into_pir(tokens);
        Ok(pir)
    }
}

pub mod files {

    use std::{fmt::Display, path::PathBuf};

    use general::{Source, Span};

    use crate::Loader;

    use super::LoadDirective;

    pub struct FileLoader {
        lib_roots: Vec<PathBuf>,
    }

    impl FileLoader {
        /// Creates a new unconfigured FileLoader Instance
        pub fn new() -> Self {
            Self {
                lib_roots: Vec::new(),
            }
        }

        /// Adds a new Library Root to the List of Places to search through when including a File
        pub fn add_lib_root(&mut self, path: PathBuf) {
            self.lib_roots.push(path);
        }
    }

    impl Default for FileLoader {
        fn default() -> Self {
            Self::new()
        }
    }

    #[derive(Debug)]
    pub struct FileLoadError {
        target: LoadDirective,
    }
    impl Display for FileLoadError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "File-Loading-Error [target = {:?}]", self.target)
        }
    }
    impl std::error::Error for FileLoadError {}

    impl Loader for FileLoader {
        type LoadError = FileLoadError;

        fn load_file(&self, path: LoadDirective) -> Result<general::Span, Self::LoadError> {
            let roots = {
                let initial = match &path.local_root {
                    Some(root) => vec![root.clone()],
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
                let res_source = Source::new(path_str, content);
                let res_span: Span = res_source.into();

                return Ok(res_span);
            }

            Err(FileLoadError { target: path })
        }
    }
}
