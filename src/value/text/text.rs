use crate::text::{Character, Encoding, NewTextError, TextSlice};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

pub struct Text<E>(Arc<TextSlice<E>>, PhantomData<E>);

impl<E> Clone for Text<E> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), self.1)
	}
}

impl<E> Debug for Text<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&**self, f)
	}
}

impl<E> Eq for Text<E> {}
impl<E> PartialEq for Text<E> {
	fn eq(&self, rhs: &Self) -> bool {
		**self == **rhs
	}
}

impl<E> PartialOrd for Text<E> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(rhs))
	}
}
impl<E> Ord for Text<E> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		(**self).cmp(&**rhs)
	}
}
impl<E> Hash for Text<E> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		(**self).hash(state)
	}
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Text: Send, Sync);

impl<E> Default for Text<E> {
	#[inline]
	fn default() -> Self {
		<&TextSlice<E>>::default().into()
	}
}

impl<E> Display for Text<E> {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl<E> std::ops::Deref for Text<E> {
	type Target = TextSlice<E>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<E> From<Box<TextSlice<E>>> for Text<E> {
	#[inline]
	fn from(text: Box<TextSlice<E>>) -> Self {
		Self(text.into(), PhantomData)
	}
}

impl<E: Encoding> TryFrom<String> for Text<E> {
	type Error = NewTextError;

	fn try_from(inp: String) -> Result<Self, Self::Error> {
		let boxed = Box::<TextSlice<E>>::try_from(inp.into_boxed_str())?;

		Ok(Self(boxed.into(), PhantomData))
	}
}

impl<E> From<Character<E>> for Text<E> {
	fn from(inp: Character<E>) -> Self {
		Self::new(inp).unwrap()
	}
}

impl<E> Text<E> {
	pub fn builder() -> super::Builder<E> {
		Default::default()
	}

	pub fn new(inp: impl ToString) -> Result<Self, NewTextError> {
		inp.to_string().try_into()
	}
}

impl<E> std::borrow::Borrow<TextSlice<E>> for Text<E> {
	fn borrow(&self) -> &TextSlice<E> {
		self
	}
}

impl<E> From<&TextSlice<E>> for Text<E> {
	fn from(text: &TextSlice<E>) -> Self {
		Self(Arc::from(text.to_string().into_boxed_str()), PhantomData)
	}
}

impl<E> TryFrom<&str> for Text<E> {
	type Error = NewTextError;

	#[inline]
	fn try_from(inp: &str) -> Result<Self, Self::Error> {
		<&TextSlice<E>>::try_from(inp).map(From::from)
	}
}
