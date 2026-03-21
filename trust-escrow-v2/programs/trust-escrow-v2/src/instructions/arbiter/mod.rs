//! Arbiter instructions module

pub mod register_arbiters;
pub mod raise_dispute;
pub mod resolve_dispute;

pub use register_arbiters::RegisterArbiters;
pub use raise_dispute::RaiseDispute;
pub use resolve_dispute::ResolveDispute;