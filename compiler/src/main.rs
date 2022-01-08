use compiler::run;
use preprocessor::loader::files::FileLoader;

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

pub fn get_cli(mut args: Vec<String>) -> (FileLoader, Vec<String>) {
    let source_file = args.pop().unwrap();
    let sources = get_sources(source_file);

    let mut loader = FileLoader::new();

    for arg in args {
        if let Some(path) = arg.strip_prefix("-L") {
            loader.add_lib_root(std::path::Path::new(path).to_path_buf());
        }
    }

    (loader, sources)
}

fn main() {
    let mut args_iter = std::env::args();
    args_iter.next();
    let args: Vec<_> = args_iter.collect();
    dbg!(&args);

    let (loader, sources) = get_cli(args);

    match run(sources, loader) {
        Ok(_) => {
            println!("Compiled Program");
        }
        Err(e) => {
            e.display();
        }
    };
}
