use std::{path::PathBuf, str::FromStr};

use compiler::run;
use preprocessor::loader::files::FileLoader;

fn main() {
    let source_file = "./tests/files/final.c";

    let mut loader = FileLoader::new();
    loader.add_lib_root(
        PathBuf::from_str("/Library/Developer/CommandLineTools/usr/include/c++/v1/").unwrap(),
    );

    run(source_file, loader);
}
