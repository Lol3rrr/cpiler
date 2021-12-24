//! This module contains all the available optimizations currently implemented

mod merger;
pub use merger::*;

mod deadcode;
pub use deadcode::*;
