//! Job instructions module

pub mod create_job;
pub mod deposit_funds;
pub mod accept_job;
pub mod submit_work;
pub mod approve_work;
pub mod reject_work;
pub mod cancel_job;

pub use create_job::CreateJob;
pub use deposit_funds::DepositFunds;
pub use accept_job::AcceptJob;
pub use submit_work::SubmitWork;
pub use approve_work::ApproveWork;
pub use reject_work::RejectWork;
pub use cancel_job::CancelJob;