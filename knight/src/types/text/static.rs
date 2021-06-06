use super::{TextInner, Text};
use std::borrow::Borrow;

/// A struct that represents a Knight text with a static lifetime.
///
/// Note that this struct is intended to live for the lifetime of the program, and should be used as such.
#[repr(transparent)]
pub struct TextStatic(TextInner);

// TODO: explain safety
unsafe impl Send for TextStatic {}
unsafe impl Sync for TextStatic {}

impl TextStatic {
	#[inline]
	pub const unsafe fn new_unchecked(data: &'static str) -> Self {
		Self(TextInner::new_static_from_str_unchecked(data))
	}
}

impl Borrow<Text> for &'static TextStatic {
	#[inline]
	fn borrow(&self) -> &'static Text {
		unsafe {
			&*(self as *const &'static TextStatic as *const Text)
		}
	}
}

impl AsRef<Text> for &'static TextStatic {
	fn as_ref(&self) -> &Text {
		self.borrow()
	}
}
