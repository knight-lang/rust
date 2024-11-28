mod ascii;
mod knight;
mod utf8;

#[derive(Debug)]
pub enum EncodingError {}

pub trait Encoding {
	fn validate(source: &str) -> Result<(), EncodingError>;
}

#[repr(transparent)]
pub struct StringSlice<T: Encoding> {
	_encoding: std::marker::PhantomData<T>,
	string: str,
}

impl<T: Encoding> StringSlice<T> {
	pub const unsafe fn new_unchecked(source: &str) -> &Self {
		if cfg!(debug_assertions) {
			T::validate(source).expect("new_unchecked failed");
		}

		// SAFETY: `StringSlice` has the same layout as `str`.
		unsafe { &*(source as *const str as *const Self) }
	}

	pub const fn new(source: &str) -> Result<&Self, EncodingError> {
		if let Err(err) = T::validate(source) {
			return Err(err);
		}

		// SAFETY: we just made sure it was a valid encoding
		Ok(unsafe { Self::new_unchecked(source) })
	}
}
