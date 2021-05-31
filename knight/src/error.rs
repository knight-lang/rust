#[derive(Debug)]
pub enum Error {
	UndefinedVariable(Box<str>),
	NumberOverflow,
	UndefinedConversion { from: &'static str, into: &'static str }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<crate::text::NumberOverflow> for Error {
	#[inline]
	fn from(_: crate::text::NumberOverflow) -> Self {
		todo!()
	}
}