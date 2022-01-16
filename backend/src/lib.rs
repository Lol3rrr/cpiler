#![allow(clippy::upper_case_acronyms)]

mod backends;
mod isas;
mod util;

pub struct Config {
    target: general::arch::Target,
}

pub fn codegen(program: ir::Program, conf: Config) {
    let target = backends::get_backend(&conf);

    target.generate(program);
}

impl Config {
    pub fn new(target: general::arch::Target) -> Self {
        Self { target }
    }
}
