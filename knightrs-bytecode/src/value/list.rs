use crate::parser::{ParseError, ParseErrorKind, Parseable, Parser};
use crate::program::{Compilable, Compiler};
use crate::strings::StringSlice;
use crate::value::{Boolean, Integer, KString, NamedType, ToBoolean, ToInteger, ToKString, Value};
use crate::{Environment, Error, Options};
use std::slice::Iter;

/// A List represents a list of [`Value`]s within Knight.
// todo: optimize me!
#[derive(Debug, Clone, PartialEq)]
pub struct List(Option<Box<[Value]>>);

/// Represents the ability to be converted to a [`List`].
pub trait ToList {
	/// Converts `self` to a [`List`].
	fn to_list(&self, env: &mut Environment) -> crate::Result<List>;
}

impl NamedType for List {
	#[inline]
	fn type_name(&self) -> &'static str {
		"List"
	}
}

impl Default for List {
	#[inline]
	fn default() -> Self {
		Self(None)
	}
}

impl ToBoolean for List {
	#[inline]
	fn to_boolean(&self, _: &mut Environment) -> crate::Result<Boolean> {
		Ok(!self.is_empty())
	}
}

impl ToKString for List {
	#[inline]
	fn to_kstring(&self, env: &mut Environment) -> crate::Result<KString> {
		// COMPLIANCE: `\n` is always a valid string character.
		static NEWLINE: &'static StringSlice = StringSlice::new_unvalidated("\n");

		self.join(&NEWLINE, env)
	}
}

impl ToInteger for List {
	fn to_integer(&self, env: &mut Environment) -> crate::Result<Integer> {
		// Note we never need to check for len -> i64 bounds, as we're guaranteed by Rust that our
		// underlying `Box<[Value]>` can hold no more than `isize::MAX`, which at worst is `i64::MAX`.
		// Thus, we can safely convert between the two without worrying about truncation.

		// If `check_container_length` is enabled, then any list's length can already be represented
		// by an `Integer`. It's only if we aren't checking list length but _are_ checking integer
		// bounds that we need this check.
		#[cfg(feature = "compliance")]
		if !env.opts().compliance.check_container_length && env.opts().compliance.i32_integer {
			return Ok(Integer::new_error(self.len() as i64, env.opts())?);
		}

		Ok(Integer::new(self.len() as i64, env.opts()).expect("(this will never fail)"))
	}
}

impl ToList for List {
	/// Simply returns `self`
	#[inline]
	fn to_list(&self, _: &mut Environment) -> crate::Result<List> {
		Ok(self.clone())
	}
}

/// TODO
#[derive(Clone)]
pub struct ListRefIter<'a>(Iter<'a, Value>);
impl<'a> Iterator for ListRefIter<'a> {
	type Item = &'a Value;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

impl<'a> IntoIterator for &'a List {
	type Item = &'a Value;
	type IntoIter = ListRefIter<'a>;
	fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
		self.iter()
	}
}

impl List {
	/// The maximum length a list can be when compliance checking is enabled.
	pub const COMPLIANCE_MAX_LEN: usize = i32::MAX as usize;

	/// Creates a new [`List`] from the given `iter`, with the given options.
	pub fn new(iter: impl IntoIterator<Item = Value>, opts: &Options) -> crate::Result<Self> {
		let v = iter.into_iter().collect::<Vec<_>>();

		#[cfg(feature = "compliance")]
		if opts.compliance.check_container_length && Self::COMPLIANCE_MAX_LEN < v.len() {
			return Err(Error::ListIsTooLarge);
		}

		Ok(Self::_new(v))
	}

	/// Creates a new `list` from `slice`, without ensuring its length is correct.
	///
	/// # Compliance
	/// Callers to this must ensure that `iter` will always have fewer than
	/// [`List::COMPLIANCE_MAX_LEN`] elements.
	pub fn new_unvalidated(iter: impl IntoIterator<Item = Value>) -> Self {
		let v = iter.into_iter().collect::<Vec<_>>();

		debug_assert!(v.len() <= Self::COMPLIANCE_MAX_LEN);

		Self::_new(v)
	}

	fn _new(vec: Vec<Value>) -> Self {
		if vec.len() == 0 {
			Self(None)
		} else {
			Self(Some(vec.into()))
		}
	}

	/// Returns a new [`List`] with the only element being `value`.
	pub fn boxed(value: Value) -> Self {
		// COMPLIANCE: We always have exactly 1 element, which is within bounds.
		Self::new_unvalidated([value])
	}

	/// Returns whether `self` is empty.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.0.is_none()
	}

	/// Gets the length of `self`.
	pub fn len(&self) -> usize {
		self.0.as_ref().map_or(0, |c| c.len())
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

	#[deprecated(note = "is this ever used?")]
	pub fn try_get<'a, F: ListGet<'a>>(&'a self, index: F) -> crate::Result<F::Output> {
		let last_index = index.last_index();
		self.get(index).ok_or(Error::IndexOutOfBounds { len: self.len(), index: last_index })
	}

	/// Returns a new list with both `self` and `rhs` concatenated.
	///
	/// # Errors
	/// If `container-length-limit` not enabled, this method will never fail. I fit is, and
	/// [`List::MAX_LEN`] is smaller than `self.len() + rhs.len()`, then an [`Error::DomainError`] is
	/// returned.
	pub fn concat(&self, rhs: &Self, opts: &Options) -> crate::Result<Self> {
		if self.is_empty() {
			return Ok(rhs.clone());
		}

		if rhs.is_empty() {
			return Ok(self.clone());
		}

		// TODO: should we do a check for length before doing this?

		Self::new(self.iter().cloned().chain(rhs.into_iter().cloned()), opts)
	}

	/// Returns a new list where `self` is repeated `amount` times.
	///
	/// This will return `None` if `self.len() * amount` is greater than [`Integer::MAX`].
	///
	/// # Errors
	/// If `container-length-limit` is not enabled, this method will never fail. If it is, and
	/// [`List::MAX_LEN`] is smaller than `self.len() * amount`, then a [`Error::DomainError`] is
	/// returned.
	pub fn repeat(&self, amount: usize, opts: &Options) -> crate::Result<Self> {
		if self.is_empty() {
			return Ok(self.clone());
		}

		#[cfg(feature = "compliance")]
		if opts.compliance.check_container_length
			&& self.len().checked_mul(amount).map_or(true, |x| Self::COMPLIANCE_MAX_LEN < x)
		{
			return Err(Error::DomainError("length of repetition is out of bounds"));
		}

		match amount {
			0 => Ok(Self::default()),
			1 => Ok(self.clone()),
			_ => Self::new(self.iter().cycle().cloned().take(self.len() * amount), opts),
		}
	}

	/// Converts each element of `self` to a string,and inserts `sep` between them.
	///
	/// # Errors
	/// Any errors that occur when converting elements to a string are returned.
	pub fn join(&self, sep: &StringSlice, env: &mut Environment) -> crate::Result<KString> {
		if self.is_empty() {
			return Ok(KString::default());
		}

		let mut joined = String::new();

		let mut is_first = true;
		for ele in self {
			if is_first {
				is_first = false;
			} else {
				joined.push_str(sep.as_str());
			}
			joined.push_str(&ele.to_kstring(env)?.as_str());
		}

		KString::new(joined, env.opts()).map_err(From::from)
	}

	/// Returns an [`ListRefIter`] instance, which iterates over borrowed references.
	pub fn iter(&self) -> ListRefIter<'_> {
		ListRefIter(self.0.as_ref().map(|x| x.iter()).unwrap_or_default())
		// ListRefIter(match self.inner() {
		// 	None => IterInner::Empty,
		// 	Some(Inner::Boxed(val)) => IterInner::Boxed(val),
		// 	Some(Inner::Slice(slice)) => IterInner::Slice(slice.iter()),
		// 	Some(Inner::Cons(lhs, rhs)) => IterInner::Cons(lhs.iter().into(), rhs),
		// 	Some(Inner::Repeat(list, amount)) => {
		// 		IterInner::Repeat(Box::new(list.iter()).cycle().take(list.len() * *amount))
		// 	}
		// })
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
		if list.is_empty() {
			return None;
		}

		return list.0.as_ref().unwrap().get(self);
		// match list.inner()? {
		// 	Inner::Boxed(ele) => (self == 0).then_some(ele),
		// 	Inner::Slice(slice) => slice.get(self),
		// 	Inner::Cons(lhs, _) if self < lhs.len() => lhs.get(self),
		// 	Inner::Cons(lhs, rhs) => rhs.get(self - lhs.len()),
		// 	Inner::Repeat(list, amount) if list.len() * amount < self => None,
		// 	Inner::Repeat(list, amount) => list.get(self % amount),
		// }
	}

	fn last_index(&self) -> usize {
		*self
	}
}

impl ListGet<'_> for std::ops::Range<usize> {
	type Output = List;

	fn get(self, list: &List) -> Option<Self::Output> {
		if list.len() < self.end || self.end < self.start {
			return None;
		}

		// FIXME: use optimizations, including maybe a "sublist" variant?
		let sublist =
			list.iter().skip(self.start).take(self.end - self.start).cloned().collect::<Vec<_>>();

		// it's a sublist, so no need to check for length
		Some(List::new_unvalidated(sublist))
	}

	fn last_index(&self) -> usize {
		self.end
	}
}

impl ListGet<'_> for std::ops::RangeFrom<usize> {
	type Output = List;

	fn get(self, list: &List) -> Option<Self::Output> {
		if list.len() < self.start {
			return None;
		}

		// FIXME: use optimizations
		let sublist = list.iter().skip(self.start).cloned().collect::<Vec<_>>();

		// SAFETY: it's a sublist, so no need to check for length
		Some(List::new_unvalidated(sublist))
	}

	fn last_index(&self) -> usize {
		self.start
	}
}

#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
impl List {
	/// Returns true if `self` contains `value`.
	pub fn contains(&self, value: &Value) -> bool {
		todo!()

		// match self.inner() {
		// 	None => false,
		// 	Some(Inner::Boxed(val)) => val == value,
		// 	Some(Inner::Slice(slice)) => slice.contains(value),
		// 	Some(Inner::Cons(lhs, rhs)) => lhs.contains(value) || rhs.contains(value),
		// 	Some(Inner::Repeat(list, _)) => list.contains(value),
		// }
	}

	/// Returns a new [`List`], deduping `self` and removing elements that exist in `rhs` as well.
	pub fn difference(&self, rhs: &Self) -> crate::Result<Self> {
		todo!()

		// let mut list = Vec::with_capacity(self.len() - rhs.len()); // arbitrary capacity.

		// for ele in self {
		// 	if !rhs.contains(ele) && !list.contains(ele) {
		// 		list.push(ele.clone());
		// 	}
		// }

		// Ok(unsafe { Self::new_unchecked(list) })
	}

	/// Returns a new list with element mapped to the return value of `block`.
	///
	/// More specifically, the variable `_` is assigned to each element, and then `block` is called.
	///
	/// # Errors
	/// Returns any errors that [`block.run`](Value::run) returns.
	pub fn map(&self, block: &Value, env: &mut Environment) -> crate::Result<Self> {
		todo!()

		// let underscore = unsafe { TextSlice::new_unchecked("_") };

		// let arg = env.lookup(underscore).unwrap();
		// let mut list = Vec::with_capacity(self.len());

		// for ele in self {
		// 	arg.assign(ele.clone());
		// 	list.push(block.run(env)?);
		// }

		// Ok(unsafe { Self::new_unchecked(list) })
	}

	/// Returns a new list where only elements for which `block` returns true are kept.
	///
	/// More specifically, the variable `_` is assigned to each element, `block` is called, and then
	/// its return value is used to check to see if the element should be kept.
	///
	/// # Errors
	/// Returns any errors that [`block.run`](Value::run) returns.
	pub fn filter(&self, block: &Value, env: &mut Environment) -> crate::Result<Self> {
		todo!()

		// let underscore = unsafe { TextSlice::new_unchecked("_") };

		// let arg = env.lookup(underscore).unwrap();
		// let mut list = Vec::with_capacity(self.len() / 2); // an arbitrary capacity constant.

		// for ele in self {
		// 	arg.assign(ele.clone());

		// 	if block.run(env)?.to_boolean(env)? {
		// 		list.push(ele.clone());
		// 	}
		// }

		// Ok(unsafe { Self::new_unchecked(list) })
	}

	/// Returns a reduction of `self` to a single element, or [`Value::Null`] if `self` is empty.
	///
	/// More specifically, the variable `a` is assigned to the first element. Then, for each other
	/// element, it is assigned to the variable `_`, and `block` is called. The return value is then
	/// assigned to `a`. After exhausting `self`, `a`'s value is returned.
	///
	/// # Errors
	/// Returns any errors that [`block.run`](Value::run) returns.
	pub fn reduce(&self, block: &Value, env: &mut Environment) -> crate::Result<Option<Value>> {
		todo!()

		// let underscore = unsafe { TextSlice::new_unchecked("_") };
		// let accumulate = unsafe { TextSlice::new_unchecked("a") };

		// let mut iter = self.iter();
		// let acc = env.lookup(accumulate).unwrap();

		// if let Some(init) = iter.next() {
		// 	acc.assign(init.clone());
		// } else {
		// 	return Ok(None);
		// }

		// let arg = env.lookup(underscore).unwrap();
		// for ele in iter {
		// 	arg.assign(ele.clone());
		// 	acc.assign(block.run(env)?);
		// }

		// Ok(Some(acc.fetch().unwrap()))
	}

	pub fn reverse(&self) -> Self {
		todo!()
		// let mut new = self.into_iter().cloned().collect::<Vec<_>>();
		// new.reverse();

		// unsafe { Self::new_unchecked(new) }
	}
}

impl<'path> Parseable<'path> for List {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, 'path>) -> Result<Option<Self::Output>, ParseError<'path>> {
		#[cfg(feature = "extensions")]
		if parser.opts().extensions.syntax.list_literals && parser.advance_if('{').is_some() {
			// TODO: make sure that this doesn't actually strictly return a list, as that won't be
			// compilable all the time (eg when `{DUMP 3}`)
			todo!("list literals")
		}

		if parser.advance_if('@').is_none() {
			return Ok(None);
		}

		Ok(Some(Self::default()))
	}
}

unsafe impl<'path> Compilable<'path> for List {
	fn compile(self, compiler: &mut Compiler<'path>, _: &Options) -> Result<(), ParseError<'path>> {
		compiler.push_constant(self.into());
		Ok(())
	}
}
