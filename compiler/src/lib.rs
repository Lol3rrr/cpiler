use std::{path::PathBuf, sync::Arc};

use optimizer::Optimization;
use preprocessor::Loader;

mod error;
pub use error::Error;

pub struct Config {
    pub target: general::arch::Target,
    pub target_file: Option<String>,
    pub build_dir: PathBuf,
    pub opt_level: u8,
}

pub fn run<L>(files: Vec<String>, loader: L, config: Config) -> Result<(), Error<L::LoadError>>
where
    L: Loader + 'static,
{
    let loader = Arc::new(loader);
    let mut irs_iter = files.into_iter().map(|src_file| {
        let preprocessed =
            preprocessor::preprocess(loader.clone(), &src_file).map_err(Error::Preprocessor)?;

        let basic_ast = syntax::parse(preprocessed).map_err(Error::Syntax)?;

        let aast = semantic::parse(basic_ast).map_err(Error::Semantic)?;

        let raw_ir = aast.convert_to_ir(config.target.0.clone());

        Ok(raw_ir)
    });

    let raw_ir = {
        let mut tmp: ir::Program = irs_iter.next().unwrap()?;

        for o in irs_iter {
            let other = o?;

            let mut prev_global_statements = tmp.global.get_statements();
            let other_global_statements = other.global.get_statements();
            prev_global_statements.extend(other_global_statements);
            tmp.global.set_statements(prev_global_statements);

            tmp.functions.extend(other.functions);
        }

        tmp
    };

    let mut optimizier_config = optimizer::Config::new();
    if config.opt_level > 0 {
        optimizier_config.add_pass(optimizer::optimizations::Merger::new());

        let chain = optimizer::optimizations::ConstantProp::new()
            .chain(optimizer::optimizations::DeadCode::new())
            .repeat(25);
        optimizier_config.add_pass(chain);
    }

    let ir = optimizer::optimize(raw_ir, optimizier_config);

    std::fs::write("./program.dot", ir.to_dot()).expect("");

    let mut backend_config = backend::Config::new(config.target.clone());
    if let Some(path) = config.target_file {
        backend_config.set_target_file(path);
    }
    backend_config.set_build_dir(config.build_dir);
    backend::codegen(ir, backend_config);

    Ok(())
}
