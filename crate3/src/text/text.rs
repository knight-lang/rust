use super::{TextInner, InvalidChar, validate};
use std::marker::PhantomData;
use std::sync::Arc;
use std::fmt::{self, Debug, Display, Formatter};
use std::convert::TryFrom;
use std::borrow::Borrow;
use std::ops::Deref;

#[repr(transparent)]
pub struct Text(*const TextInner);

impl Clone for Text {
	fn clone(&self) -> Self {
		if let TextInner::Arc(arc) = self.inner() {
			std::mem::forget(arc.clone());
		}

		Self(self.0)
	}
}

impl Drop for Text {
	fn drop(&mut self) {
		unsafe {
			// (self.0 as *mut TextInner).drop_in_place()
		}
	}
}

impl Text {
	fn inner(&self) -> &TextInner {
		unsafe {
			&*self.0
		}
	}

	pub fn new(data: impl Borrow<str> + ToString) -> Result<Self, InvalidChar> {
		validate(data.borrow())?;

		let data = data.to_string().into_boxed_str();
		let inner = Box::new(TextInner::Arc(Arc::from(data)));

		Ok(Self(Box::into_raw(inner)))
	}

	pub fn new_owned(data: String) -> Result<Self, InvalidChar> {
		Self::new(data)
	}

	pub fn new_borrowed(data: &str) -> Result<Self, InvalidChar> {
		Self::new(data) // todo: borrowed data
	}

	pub(crate) fn into_raw(self) -> *const () {
		self.0 as *const ()
	}

	pub(crate) unsafe fn from_raw(raw: *const ()) -> Self {
		Self(raw as *const TextInner)
	}

	pub fn as_str(&self) -> &str {
		match self.inner() {
			TextInner::Static(str) => &str,
			TextInner::Arc(arc) => &arc
		}
	}
}

impl Debug for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_tuple("Text")
			.field(&self.as_str())
			.finish()
	}
}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.as_str(), f)
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

impl Borrow<str> for Text {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl TryFrom<String> for Text {
	type Error = InvalidChar;

	#[inline]
	fn try_from(input: String) -> Result<Self, Self::Error> {
		Self::new(input)
	}
}

impl Deref for Text {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}