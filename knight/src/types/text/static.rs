use super::{Text, InvalidText};
use super::inner::TextInner;
use super::r#ref::TextRef;
use std::borrow::{Borrow, Cow};

/// A struct that represents a Knight text with a static lifetime.
///
/// Note that this struct is intended to live for the lifetime of the program, and should be used as such.
#[repr(transparent)]
pub struct TextStatic(TextInner);

// SAFETY: No methods on `TextStatic` modify `TextInner`. Additionally, 
unsafe impl Send for TextStatic {}
unsafe impl Sync for TextStatic {}

// impl Drop for TextStatic {
// 	fn drop(&mut self) {
		
// 	}
// }

impl TextStatic {
	#[inline]
	pub const unsafe fn new_unchecked(data: &'static str) -> Self {
		Self(TextInner::new_static_from_str_unchecked(data))
	}
}

impl &'static TextStatic {
	#[inline]
	pub fn as_text(&self) -> &Text {
		unsafe {
			TextRef(&*(self as *const TextStatic as *const TextInner))
		}
	}
}

impl Borrow<Text> for &'static TextStatic {
	#[inline]
	fn borrow(&self) -> &Text {
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
