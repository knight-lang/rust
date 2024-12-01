// TODO: make this just runtime error and parse error?
#[derive(Error, Debug)]
pub enum Error {
	#[error("{0}")]
	Todo(String),

	#[error("{0}")]
	StringError(#[from] crate::strings::StringError),

	#[error("{0}")]
	IntegerError(#[from] crate::value::integer::IntegerError),

	#[error("{0}")]
	ParseError(#[from] crate::vm::ParseError),

	#[error("bad type {type_name} to function {function:?}")]
	TypeError { type_name: &'static str, function: &'static str },

	/// Indicates that either `GET` or `SET` were given an index that was out of bounds.
	#[error("end index {index} is out of bounds for length {len}")]
	IndexOutOfBounds { len: usize, index: usize },

	#[error("list is too large")]
	ListIsTooLarge,

	#[error("(quit with exit status {0})")]
	Exit(i32),

	#[error("Conversion to {to} not defined for {from}")]
	ConversionNotDefined { to: &'static str, from: &'static str },
}

pub type Result<T> = std::result::Result<T, Error>;
