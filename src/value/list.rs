use crate::env::Options;
use crate::value::{
	Boolean, Integer, NamedType, Runnable, Text, ToBoolean, ToInteger, ToText, Value,
};
use crate::{Environment, Error, RefCount, Result, TextSlice};
use std::fmt::{self, Debug, Formatter};
use std::ops::{Range, RangeFrom};

/// The list type within Knight.
///
/// Like all types within Knight, [`List`]s are immutable.
///
/// # Portability concerns and maximum size
/// According to the Knight specs, implementations only need to support lists (and strings) with a
/// maximum length of `2,147,483,647` (ie a 32 bit integer's maximum value). So, since it is
/// possible to create a list this large, or larger (eg with `* (+,1,2) 2147483647`), we need to
/// check the length.
///
/// However, since this can be a fairly significant performance penalty, this checking is disabled
/// by default. To enable it, you should enable the `container-length-limit` feature.
#[derive(Clone, Default)]
pub struct List<'e>(Option<RefCount<Inner<'e>>>);

enum Inner<'e> {
	Boxed(Value<'e>),
	Slice(Box<[Value<'e>]>),  // nonempty slice
	Cons(List<'e>, List<'e>), // neither list is empty
	Repeat(List<'e>, usize),  // the usize is >= 2
}

/// Represents the ability to be converted to a [`List`].
pub trait ToList<'e> {
	/// Converts `self` to a [`List`].
	fn to_list(&self, opts: &Options) -> Result<List<'e>>;
}

impl PartialEq for List<'_> {
	/// Checks to see if two lists are equal.
	fn eq(&self, rhs: &Self) -> bool {
		if std::ptr::eq(self, rhs) {
			return true;
		}

		if self.len() != rhs.len() {
			return false;
		}

		self.iter().zip(rhs.iter()).all(|(l, r)| l == r)
	}
}

impl Debug for List<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_list().entries(self.iter()).finish()
	}
}

impl<'e> TryFrom<Box<[Value<'e>]>> for List<'e> {
	type Error = Error;

	#[inline]
	fn try_from(list: Box<[Value<'e>]>) -> Result<Self> {
		Self::new(list)
	}
}

impl<'e> TryFrom<Vec<Value<'e>>> for List<'e> {
	type Error = Error;

	#[inline]
	fn try_from(list: Vec<Value<'e>>) -> Result<Self> {
		list.into_boxed_slice().try_into()
	}
}

impl<'e> FromIterator<Value<'e>> for Result<List<'e>> {
	fn from_iter<T: IntoIterator<Item = Value<'e>>>(iter: T) -> Self {
		iter.into_iter().collect::<Vec<Value<'e>>>().try_into()
	}
}

impl NamedType for List<'_> {
	const TYPENAME: &'static str = "List";
}

impl<'e> List<'e> {
	/// An empty [`List`].
	pub const EMPTY: Self = Self(None);

	/// The maximum length for [`List`]s. Only used when `container-length-limit` is enabled.
	pub const MAX_LEN: usize = i32::MAX as usize;

	fn _new(inner: Inner<'e>) -> Self {
		Self(Some(inner.into()))
	}

	fn inner(&self) -> Option<&Inner<'e>> {
		self.0.as_deref()
	}

	/// Creates a new `list` from `slice`.
	///
	/// # Errors
	/// If `container-length-limit` is enabled, and `slice.len()` is larger than [`List::MAX_LEN`],
	/// then an [`Error::DomainError`] is returned. If `container-length-limit` is not enabled,
	/// this function will always succeed.
	pub fn new<T: Into<Box<[Value<'e>]>>>(slice: T) -> Result<Self> {
		let slice = slice.into();

		match slice.len() {
			0 => Ok(Self::default()),
			// OPTIMIZE: is there a way to not do `.clone()`?
			1 => Ok(Self::boxed(slice[0].clone())),

			#[cfg(feature = "container-length-limit")]
			Self::MAX_LEN.. => Err(Error::DomainError("length of slice is out of bounds")),

			_ => Ok(Self::_new(Inner::Slice(slice))),
		}
	}

	/// Returns a new [`List`] with the only element being `value`.
	pub fn boxed(value: Value<'e>) -> Self {
		Self::_new(Inner::Boxed(value))
	}

	/// Returns whether `self` is empty.
	pub fn is_empty(&self) -> bool {
		// Every inner variant should be nonempty.
		debug_assert_eq!(self.0.is_none(), self.len() == 0, "nonempty variant? len={}", self.len());

		self.0.is_none()
	}

	/// Gets the length of `self`.
	pub fn len(&self) -> usize {
		match self.inner() {
			None => 0,
			Some(Inner::Boxed(_)) => 1,
			Some(Inner::Slice(slice)) => slice.len(),
			Some(Inner::Cons(lhs, rhs)) => lhs.len() + rhs.len(),
			Some(Inner::Repeat(list, amount)) => list.len() * amount,
		}
	}

	/// Returns the first element in `self`.
	pub fn head(&self) -> Option<Value<'e>> {
		self.get(0).cloned()
	}

	/// Returns everything but the first element in `self`.
	pub fn tail(&self) -> Option<Self> {
		self.get(1..)
	}

	/// Gets the value(s) at `index`.
	///
	/// This is syntactic sugar for `index.get(self)`.
	pub fn get<'a, F: ListFetch<'a, 'e>>(&'a self, index: F) -> Option<F::Output> {
		index.get(self)
	}

	/// Returns a new list with both `self` and `rhs` concatenated.
	///
	/// # Errors
	/// If `container-length-limit` not enabled, this method will never fail. I fit is, and
	/// [`List::MAX_LEN`] is smaller than `self.len() + rhs.len()`, then an [`Error::DomainError`] is
	/// returned.
	pub fn concat(&self, rhs: &Self) -> Result<Self> {
		if self.is_empty() {
			return Ok(rhs.clone());
		}

		if rhs.is_empty() {
			return Ok(self.clone());
		}

		if cfg!(feature = "container-length-limit") && Self::MAX_LEN < self.len() + rhs.len() {
			return Err(Error::DomainError("length of concatenation is out of bounds"));
		}

		Ok(Self::_new(Inner::Cons(self.clone(), rhs.clone())))
	}

	/// Returns a new list where `self` is repeated `amount` times.
	///
	/// This will return `None` if `self.len() * amount` is greater than [`Integer::MAX`].
	///
	/// # Errors
	/// If `container-length-limit` is not enabled, this method will never fail. If it is, and
	/// [`List::MAX_LEN`] is smaller than `self.len() * amount`, then a [`Error::DomainError`] is
	/// returned.
	pub fn repeat(&self, amount: usize) -> Result<Self> {
		if cfg!(feature = "container-length-limit") && Self::MAX_LEN < self.len() * amount {
			return Err(Error::DomainError("length of repetition is out of bounds"));
		}

		match amount {
			0 => Ok(Self::EMPTY),
			1 => Ok(self.clone()),
			_ => Ok(Self::_new(Inner::Repeat(self.clone(), amount))),
		}
	}

	/// Converts each element of `self` to a string,and inserts `sep` between them.
	///
	/// # Errors
	/// Any errors that occur when converting elements to a string are returned.
	pub fn join(&self, sep: &TextSlice) -> Result<Text> {
		let mut joined = Text::builder();

		let mut is_first = true;
		for ele in self {
			if is_first {
				is_first = false;
			} else {
				joined.push(sep);
			}

			joined.push(&ele.to_text()?);
		}

		Ok(joined.finish())
	}

	/// Returns an [`Iter`] instance, which iterates over borrowed references.
	pub fn iter(&self) -> Iter<'_, 'e> {
		match self.inner() {
			None => Iter::Empty,
			Some(Inner::Boxed(val)) => Iter::Boxed(val),
			Some(Inner::Slice(slice)) => Iter::Slice(slice.iter()),
			Some(Inner::Cons(lhs, rhs)) => Iter::Cons(lhs.iter().into(), rhs),
			Some(Inner::Repeat(list, amount)) => {
				Iter::Repeat(Box::new(list.iter()).cycle(), list.len() * *amount)
			}
		}
	}

	/// Returns true if `self` contains `value`.
	pub fn contains(&self, value: &Value<'e>) -> bool {
		match self.inner() {
			None => false,
			Some(Inner::Boxed(val)) => val == value,
			Some(Inner::Slice(slice)) => slice.contains(value),
			Some(Inner::Cons(lhs, rhs)) => lhs.contains(value) || rhs.contains(value),
			Some(Inner::Repeat(list, _)) => list.contains(value),
		}
	}

	/// Returns a new [`List`], deduping `self` and removing elements that exist in `rhs` as well.
	pub fn difference(&self, rhs: &Self) -> Result<Self> {
		let mut list = Vec::with_capacity(self.len() - rhs.len());

		for ele in self {
			if !rhs.contains(ele) && !list.contains(ele) {
				list.push(ele.clone());
			}
		}

		Ok(list.try_into().unwrap())
	}

	/// Returns a new list with element mapped to the return value of `block`.
	///
	/// More specifically, the variable `_` is assigned to each element, and then `block` is called.
	///
	/// # Errors
	/// Returns any errors that [`block.run`](Value::run) returns.
	pub fn map(&self, block: &Value<'e>, env: &mut Environment<'e>) -> Result<Self> {
		const UNDERSCORE: &TextSlice = unsafe { TextSlice::new_unchecked("_") };

		let arg = env.lookup(UNDERSCORE).unwrap();

		let mut mapping = Vec::with_capacity(self.len());

		for ele in self {
			arg.assign(ele.clone());
			mapping.push(block.run(env)?);
		}

		Ok(mapping.try_into().unwrap())
	}

	/// Returns a new list where only elements for which `block` returns true are kept.
	///
	/// More specifically, the variable `_` is assigned to each element, `block` is called, and then
	/// its return value is used to check to see if the element should be kept.
	///
	/// # Errors
	/// Returns any errors that [`block.run`](Value::run) returns.
	pub fn filter(&self, block: &Value<'e>, env: &mut Environment<'e>) -> Result<Self> {
		const UNDERSCORE: &TextSlice = unsafe { TextSlice::new_unchecked("_") };

		let arg = env.lookup(UNDERSCORE).unwrap();
		let mut filtering = Vec::with_capacity(self.len() / 2); // some arbitrary cap constant.

		for ele in self {
			arg.assign(ele.clone());

			if block.run(env)?.to_boolean()? {
				filtering.push(ele.clone());
			}
		}

		Ok(filtering.try_into().unwrap())
	}

	/// Returns a reduction of `self` to a single element, or [`Value::Null`] if `self` is empty.
	///
	/// More specifically, the variable `a` is assigned to the first element. Then, for each other
	/// element, it is assigned to the variable `_`, and `block` is called. The return value is then
	/// assigned to `a`. After exhausting `self`, `a`'s value is returned.
	///
	/// # Errors
	/// Returns any errors that [`block.run`](Value::run) returns.
	pub fn reduce(&self, block: &Value<'e>, env: &mut Environment<'e>) -> Result<Option<Value<'e>>> {
		const ACCUMULATE: &TextSlice = unsafe { TextSlice::new_unchecked("a") };
		const UNDERSCORE: &TextSlice = unsafe { TextSlice::new_unchecked("_") };

		let mut iter = self.iter();

		let acc = env.lookup(ACCUMULATE).unwrap();
		if let Some(init) = iter.next() {
			acc.assign(init.clone());
		} else {
			return Ok(None);
		}

		let arg = env.lookup(UNDERSCORE).unwrap();
		for ele in iter {
			arg.assign(ele.clone());
			acc.assign(block.run(env)?);
		}

		Ok(Some(acc.fetch().unwrap()))
	}
}

impl<'e> ToList<'e> for List<'e> {
	/// Simply returns `self`.
	#[inline]
	fn to_list(&self) -> Result<Self> {
		Ok(self.clone())
	}
}

impl ToBoolean for List<'_> {
	/// Returns whether `self` is nonempty.
	#[inline]
	fn to_boolean(&self) -> Result<Boolean> {
		Ok(!self.is_empty())
	}
}

impl ToInteger for List<'_> {
	/// Returns `self`'s length.
	#[inline]
	fn to_integer(&self) -> Result<Integer> {
		self.len().try_into()
	}
}

impl ToText for List<'_> {
	/// Returns `self` [joined](Self::join) with a newline.
	fn to_text(&self) -> Result<Text> {
		const NEWLINE: &TextSlice = unsafe { TextSlice::new_unchecked("\n") };

		self.join(NEWLINE)
	}
}

/// A helper trait for [`List::get`], indicating a type can index into a `List`.
pub trait ListFetch<'a, 'e> {
	/// The resulting type.
	type Output;

	/// Gets an `Output` from `list`.
	fn get(self, list: &'a List<'e>) -> Option<Self::Output>;
}

impl<'a, 'e: 'a> ListFetch<'a, 'e> for usize {
	type Output = &'a Value<'e>;

	fn get(self, list: &'a List<'e>) -> Option<Self::Output> {
		match list.inner()? {
			Inner::Boxed(ele) => (self == 0).then_some(ele),
			Inner::Slice(slice) => slice.get(self),
			Inner::Cons(lhs, rhs) => {
				if self < lhs.len() {
					lhs.get(self)
				} else {
					rhs.get(self - lhs.len())
				}
			}
			Inner::Repeat(list, amount) => {
				if list.len() * amount < self {
					None
				} else {
					list.get(self % amount)
				}
			}
		}
	}
}

impl<'e> ListFetch<'_, 'e> for Range<usize> {
	type Output = List<'e>;

	fn get(self, list: &List<'e>) -> Option<Self::Output> {
		if list.len() < self.end || self.end < self.start {
			return None;
		}

		// FIXME: use optimizations, including maybe a "sublist" variant?
		Some(
			list
				.iter()
				.skip(self.start)
				.take(self.end - self.start)
				.cloned()
				.collect::<Vec<_>>()
				.try_into()
				.unwrap(),
		)
	}
}

impl<'e> ListFetch<'_, 'e> for RangeFrom<usize> {
	type Output = List<'e>;

	fn get(self, list: &List<'e>) -> Option<Self::Output> {
		// FIXME: use optimizations
		Some(list.iter().skip(self.start).cloned().collect::<Vec<_>>().try_into().unwrap())
	}
}

impl<'a, 'e> IntoIterator for &'a List<'e> {
	type Item = &'a Value<'e>;
	type IntoIter = Iter<'a, 'e>;

	fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
		self.iter()
	}
}

/// Represents an iterator over [`List`]s.
#[derive(Clone)]
pub enum Iter<'a, 'e> {
	/// There's nothing left.
	Empty,

	/// There's only a single element to iterate over.
	Boxed(&'a Value<'e>),

	/// Iterate over the LHS elements first, then the RHS.
	Cons(Box<Self>, &'a List<'e>),

	/// Iterate over a slice of elements.
	Slice(std::slice::Iter<'a, Value<'e>>),

	/// Repeats the iterator.
	Repeat(std::iter::Cycle<Box<Self>>, usize),
}

impl<'a, 'e> Iterator for Iter<'a, 'e> {
	type Item = &'a Value<'e>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Empty => None,
			Self::Boxed(value) => {
				let ret = Some(*value);
				*self = Self::Empty;
				ret
			}
			Self::Slice(iter) => iter.next(),
			Self::Cons(iter, rhs) => {
				if let Some(value) = iter.next() {
					return Some(value);
				}

				*self = rhs.iter();
				self.next()
			}

			Self::Repeat(_, 0) => None,
			Self::Repeat(iter, n) => {
				*n -= 1;
				let value = iter.next();
				debug_assert!(value.is_some());
				value
			}
		}
	}
}
