//! phd is a small, easy-to-use Gopher server that tries to make
//! serving up a Gopher site quick and painless. Best used for local
//! development or low traffic Gopher sites.

#![allow(unused_must_use)]
#![warn(absolute_paths_not_starting_with_crate)]
#![warn(explicit_outlives_requirements)]
#![warn(unreachable_pub)]
#![warn(deprecated_in_future)]
#![warn(missing_docs)]
#![allow(clippy::while_let_on_iterator)]

pub mod color;
pub mod request;
pub mod server;

pub use crate::request::Request;

/// Alias for a generic Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
