use super::Target;
use crate::TargetConfig;

pub struct Backend {}

impl Backend {
    pub fn new() -> Self {
        Self {}
    }
}

impl Target for Backend {
    fn generate(&self, program: ir::Program, conf: TargetConfig) {
        dbg!(&conf);

        todo!("Generate")
    }
}
