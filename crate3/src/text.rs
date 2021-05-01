use std::num::NonZeroU64;
use std::sync::Arc;
use std::fmt::{self, Debug, Display, Formatter};

#[repr(transparent)]
pub struct Text(NonZeroU64);

struct Inner(Arc<str>); // todo

impl Clone for Text {
	fn clone(&self) -> Self {
		unsafe {
			std::mem::forget(self.inner().0.clone());
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

impl Text {
	fn inner_ptr(&self) -> *const Inner {
		(self.0.get() & !0b111) as *const Inner
	}

	fn inner(&self) -> &Inner {
		unsafe {
			&*self.inner_ptr()
		}
	}

	pub fn new(data: impl AsRef<str> + ToString) -> Self {
		// let data = data.to_string().into_boxed_slice();
		todo!()

		// Self(Box::into_raw(data.to_string().into_boxed_slice()))
	}

	pub const unsafe fn new_static_unchecked(data: &'static str) -> Self {

		Self(NonZeroU64::new_unchecked(1))
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
		self.inner().0.as_ref()
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
