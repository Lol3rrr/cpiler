use parking_lot::FairMutex;
use std::path::{Path, PathBuf};

use compiler::Config;
use preprocessor::loader::files::FileLoader;

static SYNC: FairMutex<()> = parking_lot::const_fair_mutex(());

macro_rules! compile_testing {
    ($name:ident, $path:expr, $compiles:expr, $ret_code:expr) => {
        #[test]
        fn $name() {
            let handle = loop {
                match SYNC.try_lock() {
                    Some(h) => break h,
                    None => {
                        std::thread::yield_now();
                        continue;
                    }
                }
            };

            let base_path = Path::new("./tests/files/basics");
            let src_path = base_path.join($path);
            let build_path = base_path.join(
                PathBuf::from(($path).to_string())
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            );

            dbg!(&src_path, &build_path);

            let loader = FileLoader::new();

            let comp_result = compiler::run(
                vec![src_path.to_str().unwrap().to_string()],
                loader,
                Config {
                    opt_level: 0,
                    target: general::arch::Target::default(),
                    target_file: Some(stringify!($name).to_string()),
                    build_dir: build_path,
                },
            );

            if $compiles {
                let _ = comp_result.unwrap();
            } else {
                let _ = comp_result.unwrap_err();
            }

            let output = match std::process::Command::new("./test").output() {
                Ok(o) => o,
                Err(e) => {
                    println!("Process: {:?}", e);
                    panic!("Failed to run Process");
                }
            };

            assert_eq!(Some($ret_code), output.status.code());

            drop(handle);
        }
    };
}

compile_testing!(address_of_array, "address_of_array.c", true, 0);
compile_testing!(array, "array.c", true, 0);
compile_testing!(basic, "basic.c", true, 0);
compile_testing!(final_, "final.c", true, 0);
compile_testing!(floats, "floats.c", true, 0);
compile_testing!(function_call, "function_call.c", true, 0);
compile_testing!(globals, "globals.c", true, 0);
compile_testing!(loops, "loops.c", true, 0);
compile_testing!(missing_include, "missing_include.c", false, 0);
compile_testing!(pointer, "pointer.c", true, 0);
compile_testing!(ptr_deref, "ptr_deref.c", true, 0);
compile_testing!(simple, "simple.c", true, 0);
compile_testing!(spill_var, "spill_vars.c", true, 0);
