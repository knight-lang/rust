use std::num::NonZeroU64;
use std::sync::Arc;
use std::fmt::{self, Debug, Display, Formatter};

#[repr(transparent)]
pub struct Text(NonZeroU64);

enum Inner {
	Static(&'static str),
	Arc(Arc<str>)
}

impl Clone for Text {
	fn clone(&self) -> Self {
		if let Inner::Arc(arc) = self.inner() {
			std::mem::forget(arc.clone());
		}

		Self(self.0)
	}
}

impl Drop for Text {
	fn drop(&mut self) {
		unsafe {
			(self.inner_ptr() as *mut Inner).drop_in_place()
		}
	}
}

#[derive(Debug)]
pub struct InvalidByte {

}

impl Text {
	fn inner_ptr(&self) -> *const Inner {
		(self.0.get() & !0b111) as *const Inner
	}

	fn inner(&self) -> &Inner {
		unsafe {
			&*self.inner_ptr()
		}
	}

	pub fn new(data: impl AsRef<str> + ToString) -> Result<Self, InvalidByte> {
		let data = data.to_string().into_boxed_str();
		let inner = Box::new(Inner::Arc(Arc::from(data)));

		unsafe {
			// cant be null pointer because its a valid address as we allocated it
			Ok(Self(NonZeroU64::new_unchecked(Box::into_raw(inner) as usize as u64)))
		}
	}

	pub fn new_borrowed(data: &str) -> Result<Self, InvalidByte> {
		Self::new(data) // todo: borrowed data
	}

	pub fn into_raw(self) -> NonZeroU64 {
		self.0
	}

	pub unsafe fn from_raw(raw: NonZeroU64) -> Self {
		Self(raw)
	}

	pub unsafe fn from_raw_ref<'a>(raw: &NonZeroU64) -> &'a Self {
		&*(raw as *const NonZeroU64 as *const Self)
	}

	pub fn as_str(&self) -> &str {
		match self.inner() {
			Inner::Static(str) => &str,
			Inner::Arc(arc) => &arc
		}
	}
}

impl Debug for Text {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		fmt.debug_tuple("Text")
			.field(&self.as_str())
			.finish()
	}
}

impl Display for Text {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.as_str(), fmt)
	}
}

impl Eq for Text {}
impl PartialEq for Text {
	fn eq(&self, rhs: &Self) -> bool {
		self.as_str() == rhs.as_str()
	}
}

impl std::hash::Hash for Text {
	fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
		self.as_str().hash(h)
	}
}

impl PartialOrd for Text {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(rhs))
	}
}

impl Ord for Text {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.as_str().cmp(rhs.as_str())
	}
}

impl AsRef<str> for Text {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl std::borrow::Borrow<str> for Text {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

pub struct TextStatic(std::mem::ManuallyDrop<Inner>);

impl TextStatic {
	pub const unsafe fn new_static_unchecked(data: &'static str) -> Self {
		Self(std::mem::ManuallyDrop::new(Inner::Static(data)))
	}

	pub fn text(&'static self) -> Text {
		unsafe {
			Text(NonZeroU64::new_unchecked(&self.0 as *const _ as usize as u64))
		}
	}
}
