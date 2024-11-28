#[derive(Error, Debug)]
pub enum Error {
	#[error("todo")]
	Todo,

	#[error("{0}")]
	StringError(#[from] crate::strings::Error),

	#[error("{0}")]
	IntegerError(#[from] crate::value::integer::Error),

	#[cfg(feature = "compliance")]
	#[error("{0} is out of bounds for integers")]
	IntegerOutOfBounds(crate::value::integer::IntegerInner),

	#[cfg(feature = "compliance")]
	#[error("integer overflow for method {0:?}")]
	IntegerOverflow(char),

	#[cfg(feature = "compliance")]
	#[error("integer overflow for method {0:?}")]
	IntegerDivisionByZero(char),
}

pub type Result<T> = std::result::Result<T, Error>;
