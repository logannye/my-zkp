//! CLI command implementations

pub mod prove;
pub mod verify;

pub use prove::{ProveArgs, prove_command};
pub use verify::{VerifyArgs, verify_command};

