use super::{validate, Character, Chars, Encoding, NewTextError, Text};
use crate::env::{Environment, Flags};
use crate::value::integer::IntType;
use crate::value::{Boolean, Integer, List, ToBoolean, ToInteger, ToList, ToText, Value};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[repr(transparent)]
pub struct TextSlice<E>(std::marker::PhantomData<E>, str);

impl<E> Debug for TextSlice<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.1, f)
	}
}
impl<E> Eq for TextSlice<E> {}
impl<E> PartialEq for TextSlice<E> {
	fn eq(&self, rhs: &Self) -> bool {
		self.1 == rhs.1
	}
}
impl<E> PartialOrd for TextSlice<E> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		self.1.partial_cmp(&rhs.1)
	}
}

impl<E> Ord for TextSlice<E> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.1.cmp(&rhs.1)
	}
}
impl<E> Hash for TextSlice<E> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.1.hash(state)
	}
}
impl<E> Default for &TextSlice<E> {
	#[inline]
	fn default() -> Self {
		// SAFETY: we know that `""` is a valid string, as it contains nothing.
		unsafe { TextSlice::new_unchecked("") }
	}
}

impl<E> Display for TextSlice<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl<E> PartialEq<str> for TextSlice<E> {
	fn eq(&self, rhs: &str) -> bool {
		&self.1 == rhs
	}
}

impl<E> Deref for TextSlice<E> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.1
	}
}

impl<E> TextSlice<E> {
	pub const MAX_LEN: usize = i32::MAX as usize;

	/// Creates a new `TextSlice` without validating `inp`.
	///
	/// # Safety
	/// - `inp` must be a valid `TextSlice`.
	pub const unsafe fn new_unchecked(inp: &str) -> &Self {
		// SAFETY: Since `TextSlice` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		&*(inp as *const str as *const Self)
	}

	pub fn new<'s>(inp: &'s str, flags: &Flags) -> Result<&'s Self, NewTextError>
	where
		E: Encoding,
	{
		validate::<E>(inp, flags).map(|_| unsafe { Self::new_unchecked(inp) })
	}

	pub fn new_boxed(inp: Box<str>, flags: &Flags) -> Result<Box<Self>, NewTextError>
	where
		E: Encoding,
	{
		validate::<E>(&inp, flags).map(|_| unsafe { Self::new_boxed_unchecked(inp) })
	}

	#[allow(unsafe_code)]
	pub unsafe fn new_boxed_unchecked(inp: Box<str>) -> Box<Self> {
		// SAFETY: Since `TextSlice` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		Box::from_raw(Box::into_raw(inp) as _)
	}

	pub const fn as_str(&self) -> &str {
		&self.1
	}

	pub fn chars(&self) -> Chars<'_, E> {
		Chars(std::marker::PhantomData, self.1.chars())
	}

	pub fn get<T: std::slice::SliceIndex<str, Output = str>>(&self, range: T) -> Option<&Self> {
		let substring = self.1.get(range)?;

		// SAFETY: We're getting a substring of a valid TextSlice, which thus will itself be valid.
		Some(unsafe { Self::new_unchecked(substring) })
	}

	pub fn concat(&self, rhs: &Self, flags: &Flags) -> Result<Text<E>, NewTextError>
	where
		E: Encoding,
	{
		let mut builder = super::Builder::with_capacity(self.len() + rhs.len());

		builder.push(self);
		builder.push(rhs);

		builder.finish(flags)
	}

	pub fn repeat(&self, amount: usize, flags: &Flags) -> Result<Text<E>, NewTextError>
	where
		E: Encoding,
	{
		Ok(Text::new((**self).repeat(amount), flags)?)
	}

	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn split<'e, I: IntType>(
		&self,
		sep: &Self,
		env: &mut Environment<'e, I, E>,
	) -> List<'e, I, E>
	where
		E: Encoding,
	{
		if sep.is_empty() {
			// TODO: optimize me
			return Value::<I, E>::from(self.to_owned()).to_list(env).unwrap();
		}

		let chars = (**self)
			.split(&**sep)
			.map(|x| Text::new(x, env.flags()).unwrap().into())
			.collect::<Vec<_>>();

		// SAFETY: If `self` is within the container bounds, so is the length of its chars.
		unsafe { List::new_unchecked(chars) }
	}

	pub fn ord<I: IntType>(&self) -> crate::Result<Integer<I>> {
		Integer::try_from(
			self.chars().next().ok_or(crate::Error::DomainError("empty string"))?.inner(),
		)
	}

	pub fn head(&self) -> Option<Character<E>> {
		self.chars().next()
	}

	pub fn tail(&self) -> Option<Text<E>> {
		let mut chrs = self.chars();

		if chrs.next().is_none() {
			None
		} else {
			Some(chrs.as_text().to_owned())
		}
	}

	pub fn remove_substr(&self, substr: &Self) -> Text<E> {
		let _ = substr;
		todo!();
	}
}

impl<'a, E> From<&'a TextSlice<E>> for &'a str {
	#[inline]
	fn from(text: &'a TextSlice<E>) -> Self {
		text
	}
}

impl<E> ToOwned for TextSlice<E> {
	type Owned = Text<E>;

	fn to_owned(&self) -> Self::Owned {
		self.into()
	}
}

impl<'a, E> IntoIterator for &'a TextSlice<E> {
	type Item = Character<E>;
	type IntoIter = Chars<'a, E>;

	fn into_iter(self) -> Self::IntoIter {
		self.chars()
	}
}

impl<'e, I, E> ToBoolean<'e, I, E> for Text<E> {
	fn to_boolean(&self, _: &mut Environment<'e, I, E>) -> crate::Result<Boolean> {
		Ok(!self.is_empty())
	}
}

impl<'e, I, E> ToText<'e, I, E> for Text<E> {
	fn to_text(&self, _: &mut Environment<'e, I, E>) -> crate::Result<Self> {
		Ok(self.clone())
	}
}

impl<E> crate::value::NamedType for Text<E> {
	const TYPENAME: &'static str = "Text";
}

impl<'e, I: IntType, E> ToInteger<'e, I, E> for Text<E> {
	fn to_integer(&self, _: &mut Environment<'e, I, E>) -> crate::Result<Integer<I>> {
		self.parse()
	}
}

impl<'e, I, E> ToList<'e, I, E> for Text<E> {
	fn to_list(&self, _: &mut Environment<'e, I, E>) -> crate::Result<List<'e, I, E>> {
		let chars = self.chars().map(Value::from).collect::<Vec<_>>();

		// SAFETY: If `self` is within the container bounds, so is the length of its chars.
		Ok(unsafe { List::new_unchecked(chars) })
	}
}
