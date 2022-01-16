use clap::{ArgEnum, Parser};

#[derive(Debug, Clone, ArgEnum)]
pub enum Targets {
    MacAarch64,
    Fxcg50,
}

impl From<Targets> for general::arch::Arch {
    fn from(t: Targets) -> Self {
        match t {
            Targets::MacAarch64 => Self::AArch64,
            Targets::Fxcg50 => Self::SH4A_FXCG50,
        }
    }
}

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(long = "target", arg_enum)]
    pub target: Option<Targets>,

    #[clap(short = 'L')]
    pub libs: Vec<String>,

    pub input: String,
}
