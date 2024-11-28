use super::EncodingError;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct StringSlice(str);

impl Default for &'_ StringSlice {
	fn default() -> Self {
		StringSlice::new_unchecked("")
	}
}

impl StringSlice {
	pub unsafe const fn new_unchecked(source: &str) -> Result<&StringSlice> {
		// SAFETY: `StringSlice` has the same layout as `str`.
		unsafe { &*(source as *const str as *const Self) }
	}

	pub const fn new(source: &str) -> Result<&StringSlice, EncodingError> {
		// SAFETY: `StringSlice` has the same layout as `str`.
		unsafe { &*(source as *const str as *const Self) }
	}
}
