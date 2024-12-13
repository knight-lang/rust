use crate::containers::RefCount;
use crate::env::{Environment, Flags};
use crate::parse::{self, Parsable, Parser};
use crate::value::{
	Boolean, Integer, NamedType, Text, TextSlice, ToBoolean, ToInteger, ToText, Value,
};
use crate::Result;
use crate::{value::Runnable, Error};
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Range, RangeFrom};

/// The list type within Knight.
///
/// Like all types within Knight, [`List`]s are immutable.
///
/// # Portability concerns and maximum size
/// According to the Knight specs, implementations only need to support lists (and strings) with a
/// maximum length of `2147483647` (ie [`i32::MAX`]). So, since it is
/// possible to create a list this large, or larger (eg with `* (+,1,2) 2147483647`), we need to
/// check the length.
///
/// However, since this can be a fairly significant performance penalty, this checking is disabled
/// by default. To enable it, you should enable the `container-length-limit` feature.
#[derive(Clone, Default)]
pub struct List(Option<RefCount<Inner>>);

enum Inner {
	Boxed(Value),        // a single value
	Slice(Box<[Value]>), // nonempty slice
	Cons(List, List),    // neither list is empty
	Repeat(List, usize), // the usize is >= 2
}

/// Represents the ability to be converted to a [`List`].
pub trait ToList {
	/// Converts `self` to a [`List`].
	fn to_list(&self, env: &mut Environment) -> Result<List>;
}

impl Eq for List {}
impl PartialEq for List {
	/// Checks to see if two lists are equal.
	fn eq(&self, rhs: &Self) -> bool {
		match (self.0.as_ref(), rhs.0.as_ref()) {
			(None, None) => true,
			(Some(l), Some(r)) if RefCount::ptr_eq(l, r) => true,
			_ => self.len() == rhs.len() && self.iter().eq(rhs),
		}
	}
}

impl Hash for List {
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write_usize(self.len());

		for ele in self {
			ele.hash(state);
		}
	}
}

impl Debug for List {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_list().entries(self).finish()
	}
}

impl NamedType for List {
	const TYPENAME: &'static str = "List";
}

impl List {
	fn _new(inner: Inner) -> Self {
		Self(Some(inner.into()))
	}

	fn inner(&self) -> Option<&Inner> {
		self.0.as_deref()
	}

	/// An empty [`List`].
	pub const EMPTY: Self = Self(None);

	/// The maximum length for [`List`]s. Only used when `container-length-limit` is enabled.
	pub const MAX_LEN: usize = i32::MAX as usize;

	/// Creates a new `list` from `slice`.
	///
	/// # Errors
	/// If `container-length-limit` is enabled, and `slice.len()` is larger than [`List::MAX_LEN`],
	/// then an [`Error::DomainError`] is returned. If `container-length-limit` is not enabled,
	/// this function will always succeed.
	pub fn new<T: Into<Box<[Value]>>>(slice: T, flags: &Flags) -> Result<Self> {
		let slice = slice.into();

		#[cfg(feature = "compliance")]
		if flags.compliance.check_container_length && Self::MAX_LEN < slice.len() {
			return Err(Error::DomainError("length of slice is out of bounds"));
		}

		let _ = flags;
		Ok(unsafe { Self::new_unchecked(slice) })
	}

	/// Creates a new `list` from `slice`, without ensuring its length is correct.
	pub unsafe fn new_unchecked<T: Into<Box<[Value]>>>(slice: T) -> Self {
		let slice = slice.into();

		match slice.len() {
			0 => Self::default(),
			1 => Self::boxed(Vec::from(slice).pop().unwrap()),
			_ => Self::_new(Inner::Slice(slice)),
		}
	}

	/// Returns a new [`List`] with the only element being `value`.
	#[inline]
	pub fn boxed(value: Value) -> Self {
		Self::_new(Inner::Boxed(value))
	}

	/// Returns whether `self` is empty.
	#[inline]
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
	#[inline]
	pub fn head(&self) -> Option<Value> {
		self.get(0).cloned()
	}

	/// Returns everything but the first element in `self`.
	#[inline]
	pub fn tail(&self) -> Option<Self> {
		self.get(1..)
	}

	/// Gets the value(s) at `index`.
	///
	/// This is syntactic sugar for `index.get(self)`.
	pub fn get<'a, F: ListGet<'a>>(&'a self, index: F) -> Option<F::Output> {
		index.get(self)
	}

	pub fn try_get<'a, F: ListGet<'a>>(&'a self, index: F) -> Result<F::Output> {
		let last_index = index.last_index();
		self.get(index).ok_or(Error::IndexOutOfBounds { len: self.len(), index: last_index })
	}

	/*
	/// Sets the value(s) at `index`.
	///
	/// This is syntactic sugar for `index.set(self)`.
	pub fn set<'a, F: ListSet<'a>>(&'a self, index: F, value: F::Value) {
		index.set(self, value)
	}
	*/

	/// Returns a new list with both `self` and `rhs` concatenated.
	///
	/// # Errors
	/// If `container-length-limit` not enabled, this method will never fail. I fit is, and
	/// [`List::MAX_LEN`] is smaller than `self.len() + rhs.len()`, then an [`Error::DomainError`] is
	/// returned.
	pub fn concat(&self, rhs: &Self, flags: &Flags) -> Result<Self> {
		if self.is_empty() {
			return Ok(rhs.clone());
		}

		if rhs.is_empty() {
			return Ok(self.clone());
		}

		#[cfg(feature = "compliance")]
		if flags.compliance.check_container_length && Self::MAX_LEN < self.len() + rhs.len() {
			return Err(Error::DomainError("length of concatenation is out of bounds"));
		}

		let _ = flags;
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
	pub fn repeat(&self, amount: usize, flags: &Flags) -> Result<Self> {
		#[cfg(feature = "compliance")]
		if flags.compliance.check_container_length
			&& self.len().checked_mul(amount).map_or(true, |x| Self::MAX_LEN < x)
		{
			return Err(Error::DomainError("length of repetition is out of bounds"));
		}

		let _ = flags;

		if self.is_empty() {
			return Ok(Self::EMPTY);
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
	pub fn join(&self, sep: &TextSlice, env: &mut Environment) -> Result<Text> {
		let mut joined = Text::builder();

		let mut is_first = true;
		for ele in self {
			if !is_first {
				joined.push(sep);
			}
			is_first = false;

			joined.push(&ele.to_text(env)?);
		}

		Ok(joined.finish(env.flags())?)
	}

	/// Returns an [`Iter`] instance, which iterates over borrowed references.
	pub fn iter(&self) -> Iter<'_> {
		Iter(match self.inner() {
			None => IterInner::Empty,
			Some(Inner::Boxed(val)) => IterInner::Boxed(val),
			Some(Inner::Slice(slice)) => IterInner::Slice(slice.iter()),
			Some(Inner::Cons(lhs, rhs)) => IterInner::Cons(lhs.iter().into(), rhs),
			Some(Inner::Repeat(list, amount)) => {
				IterInner::Repeat(Box::new(list.iter()).cycle().take(list.len() * *amount))
			}
		})
	}
}

#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
impl List {
	/// Returns true if `self` contains `value`.
	pub fn contains(&self, value: &Value) -> bool {
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
		let mut list = Vec::with_capacity(self.len() - rhs.len()); // arbitrary capacity.

		for ele in self {
			if !rhs.contains(ele) && !list.contains(ele) {
				list.push(ele.clone());
			}
		}

		Ok(unsafe { Self::new_unchecked(list) })
	}

	/// Returns a new list with element mapped to the return value of `block`.
	///
	/// More specifically, the variable `_` is assigned to each element, and then `block` is called.
	///
	/// # Errors
	/// Returns any errors that [`block.run`](Value::run) returns.
	pub fn map(&self, block: &Value, env: &mut Environment) -> Result<Self> {
		let underscore = unsafe { TextSlice::new_unchecked("_") };

		let arg = env.lookup(underscore).unwrap();
		let mut list = Vec::with_capacity(self.len());

		for ele in self {
			arg.assign(ele.clone());
			list.push(block.run(env)?);
		}

		Ok(unsafe { Self::new_unchecked(list) })
	}

	/// Returns a new list where only elements for which `block` returns true are kept.
	///
	/// More specifically, the variable `_` is assigned to each element, `block` is called, and then
	/// its return value is used to check to see if the element should be kept.
	///
	/// # Errors
	/// Returns any errors that [`block.run`](Value::run) returns.
	pub fn filter(&self, block: &Value, env: &mut Environment) -> Result<Self> {
		let underscore = unsafe { TextSlice::new_unchecked("_") };

		let arg = env.lookup(underscore).unwrap();
		let mut list = Vec::with_capacity(self.len() / 2); // an arbitrary capacity constant.

		for ele in self {
			arg.assign(ele.clone());

			if block.run(env)?.to_boolean(env)? {
				list.push(ele.clone());
			}
		}

		Ok(unsafe { Self::new_unchecked(list) })
	}

	/// Returns a reduction of `self` to a single element, or [`Value::Null`] if `self` is empty.
	///
	/// More specifically, the variable `a` is assigned to the first element. Then, for each other
	/// element, it is assigned to the variable `_`, and `block` is called. The return value is then
	/// assigned to `a`. After exhausting `self`, `a`'s value is returned.
	///
	/// # Errors
	/// Returns any errors that [`block.run`](Value::run) returns.
	pub fn reduce(&self, block: &Value, env: &mut Environment) -> Result<Option<Value>> {
		let underscore = unsafe { TextSlice::new_unchecked("_") };
		let accumulate = unsafe { TextSlice::new_unchecked("a") };

		let mut iter = self.iter();
		let acc = env.lookup(accumulate).unwrap();

		if let Some(init) = iter.next() {
			acc.assign(init.clone());
		} else {
			return Ok(None);
		}

		let arg = env.lookup(underscore).unwrap();
		for ele in iter {
			arg.assign(ele.clone());
			acc.assign(block.run(env)?);
		}

		Ok(Some(acc.fetch().unwrap()))
	}

	pub fn reverse(&self) -> Self {
		let mut new = self.into_iter().cloned().collect::<Vec<_>>();
		new.reverse();

		unsafe { Self::new_unchecked(new) }
	}
}

impl Parsable for List {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_>) -> parse::Result<Option<Self>> {
		if parser.advance_if('@').is_none() {
			return Ok(None);
		}

		Ok(Some(Self::default()))
	}
}

impl ToList for List {
	/// Simply returns `self`.
	#[inline]
	fn to_list(&self, _: &mut Environment) -> Result<Self> {
		Ok(self.clone())
	}
}

impl ToBoolean for List {
	/// Returns whether `self` is nonempty.
	#[inline]
	fn to_boolean(&self, _: &mut Environment) -> Result<Boolean> {
		Ok(!self.is_empty())
	}
}

impl ToInteger for List {
	/// Returns `self`'s length.
	#[inline]
	fn to_integer(&self, _: &mut Environment) -> Result<Integer> {
		self.len().try_into()
	}
}

impl ToText for List {
	/// Returns `self` [joined](Self::join) with a newline.
	#[inline]
	fn to_text(&self, env: &mut Environment) -> Result<Text> {
		const NEWLINE: &TextSlice = unsafe { TextSlice::new_unchecked("\n") };

		self.join(NEWLINE, env)
	}
}

/// A helper trait for [`List::get`], indicating a type can index into a `List`.
pub trait ListGet<'a> {
	/// The resulting type.
	type Output;

	/// Gets an `Output` from `list`.
	fn get(self, list: &'a List) -> Option<Self::Output>;

	fn last_index(&self) -> usize;
}

impl<'a> ListGet<'a> for usize {
	type Output = &'a Value;

	fn get(self, list: &'a List) -> Option<Self::Output> {
		match list.inner()? {
			Inner::Boxed(ele) => (self == 0).then_some(ele),
			Inner::Slice(slice) => slice.get(self),
			Inner::Cons(lhs, _) if self < lhs.len() => lhs.get(self),
			Inner::Cons(lhs, rhs) => rhs.get(self - lhs.len()),
			Inner::Repeat(list, amount) if list.len() * amount < self => None,
			Inner::Repeat(list, amount) => list.get(self % amount),
		}
	}

	fn last_index(&self) -> usize {
		*self
	}
}

impl ListGet<'_> for Range<usize> {
	type Output = List;

	fn get(self, list: &List) -> Option<Self::Output> {
		if list.len() < self.end || self.end < self.start {
			return None;
		}

		// FIXME: use optimizations, including maybe a "sublist" variant?
		let sublist =
			list.iter().skip(self.start).take(self.end - self.start).cloned().collect::<Vec<_>>();

		// SAFETY: it's a sublist, so no need to check for length
		Some(unsafe { List::new_unchecked(sublist) })
	}

	fn last_index(&self) -> usize {
		self.end
	}
}

impl ListGet<'_> for RangeFrom<usize> {
	type Output = List;

	fn get(self, list: &List) -> Option<Self::Output> {
		if list.len() < self.start {
			return None;
		}

		// FIXME: use optimizations
		let sublist = list.iter().skip(self.start).cloned().collect::<Vec<_>>();

		// SAFETY: it's a sublist, so no need to check for length
		Some(unsafe { List::new_unchecked(sublist) })
	}

	fn last_index(&self) -> usize {
		self.start
	}
}
/*
/// A helper trait for [`List::Set`], indicating a type can assign into a `List`.
pub trait ListSet<'a> {
	/// The type to use when assigning.
	type Value;

	/// Sets an `Output` from `list`.
	fn set(self, list: &'a List, value: Value);
}

impl<'a> ListSet<'a> for usize {
	type Value = Value;

	fn set(self, list: &'a List, value: Value) -> bool {
		match list.inner()? {
			Inner::Boxed(ele) => (self == 0).then_some(ele),
			Inner::Slice(slice) => slice.get(self),
			Inner::Cons(lhs, _) if self < lhs.len() => lhs.get(self),
			Inner::Cons(lhs, rhs) => rhs.get(self - lhs.len()),
			Inner::Repeat(list, amount) if list.len() * amount < self => None,
			Inner::Repeat(list, amount) => list.get(self % amount),
		}
	}
}

impl ListSet<'_> for Range<usize> {
	type Output = List;

	fn get(self, list: &List) -> Option<Self::Output> {
		if list.len() < self.end || self.end < self.start {
			return None;
		}

		// FIXME: use optimizations, including maybe a "sublist" variant?
		let sublist =
			list.iter().skip(self.start).take(self.end - self.start).cloned().collect::<Vec<_>>();

		// SAFETY: it's a sublist, so no need to check for length
		Some(unsafe { List::new_unchecked(sublist) })
	}
}

impl ListSet<'_> for RangeFrom<usize> {
	type Output = List;

	fn get(self, list: &List) -> Option<Self::Output> {
		// FIXME: use optimizations
		let sublist = list.iter().skip(self.start).cloned().collect::<Vec<_>>();

		// SAFETY: it's a sublist, so no need to check for length
		Some(unsafe { List::new_unchecked(sublist) })
	}
}*/

impl<'a> IntoIterator for &'a List {
	type Item = &'a Value;
	type IntoIter = Iter<'a>;

	#[inline]
	fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
		self.iter()
	}
}

/// Represents an iterator over [`List`]s.
#[derive(Debug, Clone)]
pub struct Iter<'a>(IterInner<'a>);

#[derive(Debug, Clone)]
enum IterInner<'a> {
	/// There's nothing left.
	Empty,

	/// There's only a single element to iterate over.
	Boxed(&'a Value),

	/// Iterate over the LHS elements first, then the RHS.
	Cons(Box<Iter<'a>>, &'a List),

	/// Iterate over a slice of elements.
	Slice(std::slice::Iter<'a, Value>),

	/// Repeats the iterator.
	Repeat(std::iter::Take<std::iter::Cycle<Box<Iter<'a>>>>),
}

impl<'a> Iterator for Iter<'a> {
	type Item = &'a Value;

	fn next(&mut self) -> Option<Self::Item> {
		match self.0 {
			IterInner::Empty => None,
			IterInner::Boxed(value) => {
				let ret = Some(value);
				self.0 = IterInner::Empty;
				ret
			}
			IterInner::Slice(ref mut iter) => iter.next(),
			IterInner::Cons(ref mut iter, rhs) => {
				if let Some(value) = iter.next() {
					return Some(value);
				}

				*self = rhs.iter();
				self.next()
			}

			IterInner::Repeat(ref mut iter) => iter.next(),
		}
	}
}
