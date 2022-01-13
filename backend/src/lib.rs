mod backends;
mod isas;
mod util;

pub struct Config {
    arch: general::arch::Arch,
}

pub fn codegen(program: ir::Program, conf: Config) {
    let target = backends::get_backend(&conf);

    target.generate(program);
}

impl Config {
    pub fn new(arch: general::arch::Arch) -> Self {
        Self { arch }
    }
}
