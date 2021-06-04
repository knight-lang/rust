use super::{Text, TextInner, InvalidSourceByte};
use std::borrow::{Borrow, Cow};

#[repr(transparent)]
pub struct TextStatic(TextInner);

impl TextStatic {
	#[inline]
	pub const fn new(data: &'static str) -> Result<Self, InvalidSourceByte> {
		unsafe {
			// todo
			Ok(Self::new_unchecked(data))
		}
	}

	#[inline]
	pub const unsafe fn new_unchecked(data: &'static str) -> Self {
		Self(TextInner {
			rc: std::sync::atomic::AtomicUsize::new(0),
			data: Cow::Borrowed(data),
			alloc: false
		})
	}

	#[inline]
	pub fn as_text(&'static self) -> Text {
		unsafe {
			std::mem::transmute::<&'static TextStatic, Text>(self)
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
