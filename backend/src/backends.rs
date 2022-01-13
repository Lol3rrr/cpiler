use general::arch::Arch;

use crate::Config;

mod aarch64_mac;
mod sh4a_fxcg50;

pub trait Target {
    fn generate(&self, program: ir::Program);
}

pub fn get_backend(config: &Config) -> Box<dyn Target> {
    match config.arch {
        Arch::AArch64 => Box::new(aarch64_mac::Backend::new()),
        Arch::SH4A_FXCG50 => Box::new(sh4a_fxcg50::Backend::new()),
        _ => todo!(),
    }
}
