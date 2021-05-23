#[cfg_attr(all(feature="unsafe-reckless", not(feature="abort-on-errors")), path="./reckless.rs")]
#[cfg_attr(any(not(feature="unsafe-reckless"), feature="abort-on-errors"), path="./normal.rs")]
mod error;

pub use error::Error;

/// A type alias for [`std::result::Result`].
pub type Result<T> = std::result::Result<T, Error>;
