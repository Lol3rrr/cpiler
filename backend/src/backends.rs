use general::arch::Arch;

use crate::Config;

mod aarch64_mac;

pub trait Target {
    fn generate(&self, program: ir::Program);
}

pub fn get_backend(config: &Config) -> Box<dyn Target> {
    match config.arch {
        Arch::AArch64 => Box::new(aarch64_mac::Backend::new()),
        _ => todo!(),
    }
}
