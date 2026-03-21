//! User instructions module
//!
//! Instructions:
//! - create_user: Create a new user account
//! - add_wallet: Add a secondary wallet
//! - set_active_wallet: Change active wallet
//! - update_user: Update user profile (bio)

pub mod create_user;
pub mod add_wallet;
pub mod set_active_wallet;
pub mod update_user;

pub use create_user::*;
pub use add_wallet::*;
pub use set_active_wallet::*;
pub use update_user::*;