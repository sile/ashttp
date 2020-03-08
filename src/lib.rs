pub use error::Error;

pub mod connection;
pub mod dispatcher;

mod error;
mod server;

pub type Result<T, E = Error> = std::result::Result<T, E>;
