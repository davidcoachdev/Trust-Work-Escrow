//! Job instructions module

pub mod create_job;
pub mod deposit_funds;
pub mod accept_job;
pub mod submit_work;
pub mod approve_work;
pub mod reject_work;
pub mod cancel_job;

pub use create_job::*;
pub use deposit_funds::*;
pub use accept_job::*;
pub use submit_work::*;
pub use approve_work::*;
pub use reject_work::*;
pub use cancel_job::*;