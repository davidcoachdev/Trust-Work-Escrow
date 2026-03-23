//! User instructions module

pub mod add_wallet;
pub mod create_user;
pub mod set_active_wallet;
pub mod update_user;

pub use add_wallet::AddWallet;
pub use create_user::CreateUser;
pub use set_active_wallet::SetActiveWallet;
pub use update_user::UpdateUser;
