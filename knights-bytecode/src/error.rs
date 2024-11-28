#[derive(Error, Debug)]
pub enum Error {
	#[error("todo")]
	Todo,

	#[error("{0}")]
	StringError(#[from] crate::strings::StringError),

	#[error("{0}")]
	IntegerError(#[from] crate::value::integer::IntegerError),
}

pub type Result<T> = std::result::Result<T, Error>;
