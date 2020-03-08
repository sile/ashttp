pub use error::Error;

pub mod connection;
pub mod dispatcher;
pub mod handler;
pub mod request;

mod error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
