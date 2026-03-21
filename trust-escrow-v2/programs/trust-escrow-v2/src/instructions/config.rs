//! Config instructions module

pub mod register_arbiters;
pub mod initialize_config;
pub mod pause;
pub mod unpause;
pub mod withdraw_treasury;

pub use register_arbiters::*;
pub use initialize_config::*;
pub use pause::*;
pub use unpause::*;
pub use withdraw_treasury::*;