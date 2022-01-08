mod arguments;
use std::collections::HashMap;

pub use arguments::*;

mod to_asm;
pub use to_asm::*;

pub mod stack;
pub mod util;

use super::{asm, ArmRegister};

pub struct Context {
    pub registers: HashMap<ir::Variable, ArmRegister>,
    pub pre_ret_instr: Vec<asm::Instruction>,
    pub var: HashMap<String, isize>,
    pub stack_allocs: HashMap<ir::Variable, isize>,
}
