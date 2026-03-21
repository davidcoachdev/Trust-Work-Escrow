//! Arbiter instructions module

pub mod raise_dispute;
pub mod resolve_dispute;
pub mod register_arbiters;

pub use raise_dispute::*;
pub use resolve_dispute::*;
pub use register_arbiters::*;