#![allow(clippy::upper_case_acronyms)]
#![warn(missing_docs)]
//! This is the Part of the Compiler responsible for converting the given IR of the entire Program
//! into the final Executable and handling all the Code-Generation

use std::{path::PathBuf, str::FromStr};

use backends::TargetConfig;

mod backends;
mod isas;
pub mod util;

/// The Config is used to specify certain Things that influence the way the Code will be generated,
/// like the Target Architecture and Platform.
pub struct Config {
    target: general::arch::Target,
    target_file: Option<String>,
    build_dir: PathBuf,
}

/// This actually performs the Code-Generation for the given Program with the given Configuration
pub fn codegen(program: ir::Program, conf: Config) {
    let target = backends::get_backend(&conf);
    let target_conf = TargetConfig {
        target_file: conf.target_file,
        build_dir: conf.build_dir,
    };

    if let Err(e) = std::fs::create_dir_all(&target_conf.build_dir) {
        panic!("Creating Build-Directory: {:?}", e);
    }

    target.generate(program, target_conf);
}

impl Config {
    /// Creates a new Configuration Instance
    pub fn new(target: general::arch::Target) -> Self {
        Self {
            target,
            target_file: None,
            build_dir: PathBuf::from_str("./c-build/").unwrap(),
        }
    }

    /// Sets the Target-File
    pub fn set_target_file(&mut self, path: String) {
        self.target_file = Some(path);
    }

    /// Sets the Build Directory
    pub fn set_build_dir(&mut self, path: PathBuf) {
        self.build_dir = path;
    }
}
