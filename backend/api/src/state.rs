//! Application state.
//!
//! Currently the state is empty because the DB layer is intentionally deferred
//! until Docker is available. The struct exists so route handlers already
//! receive a typed `State<AppState>` and future repositories can be wired here
//! without changing handler signatures.

#[derive(Clone, Debug, Default)]
pub struct AppState {}
