use general::arch::{Arch, Platform};

use crate::Config;

mod aarch64_mac;
mod sh4a_fxcg50;

pub struct TargetConfig {
    pub target_file: Option<String>,
}

pub trait Target {
    fn generate(&self, program: ir::Program, conf: TargetConfig);
}

pub fn get_backend(config: &Config) -> Box<dyn Target> {
    let target = &config.target;
    match (&target.0, &target.1) {
        (Arch::AArch64, Platform::MacOs) => Box::new(aarch64_mac::Backend::new()),
        (Arch::SH4A, Platform::CasioPrizm) => Box::new(sh4a_fxcg50::Backend::new()),
        _ => todo!(),
    }
}
