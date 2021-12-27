mod context;
pub use context::*;

mod lines;
pub use lines::*;

mod blocks;
pub use blocks::*;

/// This trait provides a more general and cleaner Interface to convert a Part of the IR to the
/// Graphviz Dot format for easy visualization
pub trait ToDot {
    /// Convert the current Item into the appropriate Dot lines and return the Name of the Node
    /// that corresponds to this Item
    fn to_dot(
        &self,
        lines: &mut dyn graphviz::Graph,
        drawn: &mut DrawnBlocks,
        ctx: &Context,
    ) -> String;

    /// The Name of the Node generated for this Entitity
    fn name(&self, ctx: &Context) -> String;
}
