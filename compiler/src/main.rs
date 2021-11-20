use std::{path::PathBuf, str::FromStr};

use compiler::run;
use preprocessor::loader::files::FileLoader;

fn main() {
    let mut args_iter = std::env::args();
    args_iter.next();
    let args: Vec<_> = args_iter.collect();
    dbg!(&args);

    let source_file = args.get(0).unwrap();

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
