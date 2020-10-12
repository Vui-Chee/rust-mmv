//! Custom macros
//!
//! To use macros inside your custom module,
//! do the following:
//!
//!   #[macro_use]
//!   mod macros;
//!   use macros::*;

mod defer;
pub use self::defer::ScopeCall;
