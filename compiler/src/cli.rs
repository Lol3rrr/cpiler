use clap::{ArgEnum, Parser};
use general::arch::{Arch, Platform};

#[derive(Debug, Clone, ArgEnum)]
pub enum Targets {
    MacAarch64,
    Fxcg50,
}

impl From<Targets> for general::arch::Target {
    fn from(t: Targets) -> Self {
        match t {
            Targets::MacAarch64 => Self(Arch::AArch64, Platform::MacOs),
            Targets::Fxcg50 => Self(Arch::SH4A, Platform::CasioPrizm),
        }
    }
}

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(long = "target", arg_enum)]
    pub target: Option<Targets>,

    #[clap(short = 'L')]
    pub libs: Vec<String>,

    #[clap(short = 'O', default_value = "0")]
    pub optimization_level: u8,

    pub input: String,
}
