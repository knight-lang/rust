use crate::text::{Character, Encoding, NewTextError, TextSlice};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

pub struct Text<E: Encoding>(Arc<TextSlice<E>>, PhantomData<E>);

impl<E: Encoding> Clone for Text<E> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), self.1)
	}
}

impl<E: Encoding> Debug for Text<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&**self, f)
	}
}

impl<E: Encoding> Eq for Text<E> {}
impl<E: Encoding> PartialEq for Text<E> {
	fn eq(&self, rhs: &Self) -> bool {
		**self == **rhs
	}
}

impl<E: Encoding> PartialOrd for Text<E> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(rhs))
	}
}
impl<E: Encoding> Ord for Text<E> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		(**self).cmp(&**rhs)
	}
}
impl<E: Encoding> Hash for Text<E> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		(**self).hash(state)
	}
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Text: Send, Sync);

impl<E: Encoding> Default for Text<E> {
	#[inline]
	fn default() -> Self {
		<&TextSlice<E>>::default().into()
	}
}

impl<E: Encoding> Display for Text<E> {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl<E: Encoding> std::ops::Deref for Text<E> {
	type Target = TextSlice<E>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<E: Encoding> From<Box<TextSlice<E>>> for Text<E> {
	#[inline]
	fn from(text: Box<TextSlice<E>>) -> Self {
		Self(text.into(), PhantomData)
	}
}

impl<E: Encoding> TryFrom<String> for Text<E> {
	type Error = NewTextError;

	fn try_from(inp: String) -> Result<Self, Self::Error> {
		Box::<TextSlice<E>>::try_from(inp.into_boxed_str())
			.map(|boxed| Self(boxed.into(), PhantomData))
	}
}

impl<E: Encoding> From<Character<E>> for Text<E> {
	fn from(chr: Character<E>) -> Self {
		unsafe { Self::new_unchecked(chr.to_string()) }
	}
}

impl<E: Encoding> Text<E> {
	pub fn builder() -> super::Builder<E> {
		Default::default()
	}

	pub unsafe fn new_unchecked(string: String) -> Self {
		Self(Arc::from(TextSlice::from_boxed_unchecked(string.into_boxed_str())), PhantomData)
	}
}

impl<E: Encoding> Text<E> {
	pub fn new(inp: impl ToString) -> Result<Self, NewTextError> {
		inp.to_string().try_into()
	}
}

impl<E: Encoding> std::borrow::Borrow<TextSlice<E>> for Text<E> {
	fn borrow(&self) -> &TextSlice<E> {
		self
	}
}

impl<E: Encoding> From<&TextSlice<E>> for Text<E> {
	fn from(text: &TextSlice<E>) -> Self {
		unsafe { Self::new_unchecked(text.to_string()) }
	}
}

impl<E: Encoding> TryFrom<&str> for Text<E> {
	type Error = NewTextError;

	#[inline]
	fn try_from(inp: &str) -> Result<Self, Self::Error> {
		<&TextSlice<E>>::try_from(inp).map(From::from)
	}
}
