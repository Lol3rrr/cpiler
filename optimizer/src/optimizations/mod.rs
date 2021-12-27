//! This module contains all the available optimizations currently implemented

mod merger;
pub use merger::*;

mod deadcode;
pub use deadcode::*;

mod constants;
pub use constants::*;

mod chain;
pub use chain::*;

mod repeat;
pub use repeat::*;
