mod context;
pub use context::*;

mod lines;
pub use lines::*;

mod blocks;
pub use blocks::*;

pub trait ToDot {
    fn to_dot(&self, lines: &mut Lines, drawn: &mut DrawnBlocks, ctx: &Context) -> String;
}
