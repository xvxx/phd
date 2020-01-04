pub mod color;
pub mod request;
pub mod server;

pub use crate::request::Request;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
