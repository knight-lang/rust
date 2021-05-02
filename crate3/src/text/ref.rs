use super::{TextInner, Text};
use std::marker::PhantomData;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::borrow::Borrow;
use std::ops::Deref;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TextRef<'a>(*const TextInner, PhantomData<&'a ()>);

impl TextRef<'_> {
	pub(crate) unsafe fn from_raw(raw: *const ()) -> Self {
		Self(raw as *const TextInner, PhantomData)
	}

	fn inner(&self) -> &TextInner {
		unsafe { &*self.0 }
	}
}

impl Debug for TextRef<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_tuple("TextRef").field(&self.as_str()).finish()
	}
}

impl Display for TextRef<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.as_str(), f)
	}
}

impl Eq for TextRef<'_> {}
impl PartialEq for TextRef<'_> {
	fn eq(&self, rhs: &Self) -> bool {
		self.as_str() == rhs.as_str()
	}
}

impl Hash for TextRef<'_> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.as_str().hash(h)
	}
}

impl PartialOrd for TextRef<'_> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(rhs))
	}
}

impl Ord for TextRef<'_> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.as_str().cmp(rhs.as_str())
	}
}

impl AsRef<str> for TextRef<'_> {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<Text> for TextRef<'_> {
	fn as_ref(&self) -> &Text {
		&self
	}
}

impl Borrow<str> for TextRef<'_> {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl Borrow<Text> for TextRef<'_> {
	fn borrow(&self) -> &Text {
		&self
	}
}

impl Deref for TextRef<'_> {
	type Target = Text;

	fn deref(&self) -> &Self::Target {
		unsafe {
			std::mem::transmute::<&*const TextInner, &Text>(&self.0)
		}
	}
}