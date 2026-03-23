//! Job instructions module

pub mod accept_job;
pub mod approve_work;
pub mod cancel_job;
pub mod create_job;
pub mod deposit_funds;
pub mod reject_work;
pub mod submit_work;

pub use accept_job::AcceptJob;
pub use approve_work::ApproveWork;
pub use cancel_job::CancelJob;
pub use create_job::CreateJob;
pub use deposit_funds::DepositFunds;
pub use reject_work::RejectWork;
pub use submit_work::SubmitWork;
