#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct StringSlice(str);

impl Default for &'_ StringSlice {
	fn default() -> Self {
		StringSlice::new("")
	}
}

impl StringSlice {
	pub const fn new(source: &str) -> &StringSlice {
		// SAFETY: `StringSlice` has the same layout as `str`.
		unsafe { &*(source as *const str as *const Self) }
	}
}
