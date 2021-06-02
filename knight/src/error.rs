use crate::number::MathError;

#[derive(Debug)]
pub enum Error {
	UndefinedVariable(Box<str>),
	NumberOverflow,
	UndefinedConversion { from: &'static str, into: &'static str },
	InvalidArgument { func: char, kind: &'static str },
	Math(MathError)
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<crate::text::NumberOverflow> for Error {
	#[inline]
	fn from(_: crate::text::NumberOverflow) -> Self {
		todo!()
	}
}

impl From<std::convert::Infallible> for Error {
	#[inline]
	fn from(err: std::convert::Infallible) -> Self {
		match err {}
	}
}

impl From<MathError> for Error {
	#[inline]
	fn from(err: MathError) -> Self {
		Self::Math(err)
	}
}