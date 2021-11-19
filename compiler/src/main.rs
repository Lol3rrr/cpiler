use std::{path::PathBuf, str::FromStr};

use ariadne::{Label, Report, ReportKind, Source};
use compiler::{run, Error};
use preprocessor::loader::files::FileLoader;
use syntax::SyntaxError;

fn main() {
    let source_file = "./tests/files/final.c";

    let mut loader = FileLoader::new();
    loader.add_lib_root(
        PathBuf::from_str("/Library/Developer/CommandLineTools/usr/include/c++/v1/").unwrap(),
    );

    match run(source_file, loader) {
        Ok(_) => {
            println!("Compiled Program");
        }
        Err(e) => {
            e.display();
        }
    };
}
