#[derive(Error, Debug)]
pub enum Error {
	#[error("todo")]
	Todo,

	#[error("{0}")]
	StringError(#[from] crate::strings::StringError),

	#[error("{0}")]
	IntegerError(#[from] crate::value::integer::IntegerError),

	#[error("bad type {type_name} to function {function:?}")]
	TypeError { type_name: &'static str, function: &'static str },
}

pub type Result<T> = std::result::Result<T, Error>;
