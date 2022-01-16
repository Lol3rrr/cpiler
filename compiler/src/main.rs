use compiler::{run, Config};
use preprocessor::loader::files::FileLoader;

use clap::Parser;

mod cli;

fn get_sources<IP>(root: IP) -> Vec<String>
where
    IP: AsRef<std::path::Path>,
{
    let root = root.as_ref();
    let meta = std::fs::metadata(&root).unwrap();

    if meta.is_file() {
        return vec![root.to_str().unwrap().to_string()];
    }

    let mut result = Vec::new();

    let read_dir = std::fs::read_dir(&root).unwrap();
    for raw_path in read_dir {
        let entry = raw_path.unwrap();
        let entry_path = entry.path();
        let entry_meta = entry.metadata().unwrap();

        if entry_meta.is_dir() {
            result.extend(get_sources(entry_path));
        } else {
            result.push(entry_path.to_str().unwrap().to_string());
        }
    }

    result.into_iter().filter(|s| s.ends_with(".c")).collect()
}

fn main() {
    let args = cli::Args::parse();
    dbg!(&args);

    let sources = {
        let root_src = args.input;
        get_sources(root_src)
    };
    let loader = {
        let mut tmp = FileLoader::new();

        for path in args.libs {
            tmp.add_lib_root(std::path::Path::new(&path).to_path_buf());
        }

        tmp
    };

    let config = Config {
        arch: args
            .target
            .map(|t| t.into())
            .unwrap_or(general::arch::Arch::AArch64),
    };

    match run(sources, loader, config) {
        Ok(_) => {
            println!("Compiled Program");
        }
        Err(e) => {
            e.display();
        }
    };
}
