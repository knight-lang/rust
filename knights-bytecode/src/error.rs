use crate::strings::StringError;

#[derive(Error, Debug)]
pub enum Error {
	#[error("todo")]
	Todo,

	#[error("{0}")]
	StringError(#[from] StringError),
}

pub type Result<T> = std::result::Result<T, Error>;
