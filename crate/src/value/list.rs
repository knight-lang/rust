use crate::value::{Boolean, Integer, KnightType, Text, ToBoolean, ToInteger, ToText, Value};
use crate::{Environment, RefCount, Result, TextSlice};
use std::fmt::{self, Debug, Formatter};
use std::ops::Range;

pub trait ToList<'e> {
	fn to_list(&self) -> Result<List<'e>>;
}

#[derive(Clone, Default)]
pub struct List<'e>(Option<RefCount<Inner<'e>>>);

enum Inner<'e> {
	Boxed(Value<'e>),
	Slice(Box<[Value<'e>]>),  // nonempty slice
	Cons(List<'e>, List<'e>), // neither list is empty
	Repeat(List<'e>, usize),  // the usize is >= 2
}

impl PartialEq for List<'_> {
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

impl<'e> From<Box<[Value<'e>]>> for List<'e> {
	fn from(list: Box<[Value<'e>]>) -> Self {
		match list.len() {
			0 => Self::default(),
			// OPTIMIZE: is there a way to not do `.clone()`?
			1 => Self::boxed(list[0].clone()),
			_ => Self::_new(Inner::Slice(list)),
		}
	}
}

impl<'e> From<Vec<Value<'e>>> for List<'e> {
	fn from(list: Vec<Value<'e>>) -> Self {
		list.into_boxed_slice().into()
	}
}

impl<'e> FromIterator<Value<'e>> for List<'e> {
	fn from_iter<T: IntoIterator<Item = Value<'e>>>(iter: T) -> Self {
		iter.into_iter().collect::<Vec<Value<'e>>>().into()
	}
}

impl<'e> KnightType<'e> for List<'e> {
	const TYPENAME: &'static str = "List";
}

impl<'e> List<'e> {
	pub const EMPTY: Self = Self(None);

	fn _new(inner: Inner<'e>) -> Self {
		Self(Some(inner.into()))
	}

	fn inner(&self) -> Option<&Inner<'e>> {
		self.0.as_deref()
	}

	pub fn boxed(value: Value<'e>) -> Self {
		Self::_new(Inner::Boxed(value))
	}

	pub fn is_empty(&self) -> bool {
		debug_assert_eq!(self.0.is_none(), self.len() == 0);

		self.0.is_none()
	}

	pub fn len(&self) -> usize {
		match self.inner() {
			None => 0,
			Some(Inner::Boxed(_)) => 1,
			Some(Inner::Slice(slice)) => slice.len(),
			Some(Inner::Cons(lhs, rhs)) => lhs.len() + rhs.len(),
			Some(Inner::Repeat(list, amount)) => list.len() * amount,
		}
	}

	pub fn get<'a, F: SliceFetch<'a, 'e>>(&'a self, index: F) -> Option<F::Output> {
		index.get(self)
	}

	pub fn concat(&self, rhs: &Self) -> Self {
		if self.is_empty() {
			return rhs.clone();
		}

		if rhs.is_empty() {
			return self.clone();
		}

		Self::_new(Inner::Cons(self.clone(), rhs.clone()))
	}

	pub fn repeat(&self, amount: usize) -> Self {
		match amount {
			0 => Self::default(),
			1 => self.clone(),
			_ => Self::_new(Inner::Repeat(self.clone(), amount)),
		}
	}

	pub fn join(&self, sep: &TextSlice) -> Result<Text> {
		let mut joined = Text::builder();

		let mut is_first = true;
		for ele in self {
			if is_first {
				is_first = false;
			} else {
				joined.push(&sep);
			}

			joined.push(&ele.to_text()?);
		}

		Ok(joined.finish())
	}

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

	pub fn contains(&self, value: &Value<'e>) -> bool {
		match self.inner() {
			None => false,
			Some(Inner::Boxed(val)) => val == value,
			Some(Inner::Slice(slice)) => slice.contains(value),
			Some(Inner::Cons(lhs, rhs)) => lhs.contains(value) || rhs.contains(value),
			Some(Inner::Repeat(list, _)) => list.contains(value),
		}
	}

	#[cfg(feature = "list-extensions")]
	pub fn difference(&self, rhs: &Self) -> Self {
		let mut list = Vec::with_capacity(self.len() - rhs.len());

		for ele in self {
			if !rhs.contains(ele) && !list.contains(ele) {
				list.push(ele.clone());
			}
		}

		list.into()
	}

	#[cfg(feature = "list-extensions")]
	pub fn map(&self, block: &Value<'e>, env: &mut Environment<'e>) -> Result<Self> {
		const UNDERSCORE: &'static TextSlice = unsafe { TextSlice::new_unchecked("_") };

		let arg = env.lookup(UNDERSCORE).unwrap();

		self
			.iter()
			.map(|ele| {
				arg.assign(ele.clone());
				block.run(env)
			})
			.collect()
	}

	#[cfg(feature = "list-extensions")]
	pub fn reduce(&self, block: &Value<'e>, env: &mut Environment<'e>) -> Result<Option<Value<'e>>> {
		const ACCUMULATE: &'static TextSlice = unsafe { TextSlice::new_unchecked("a") };
		const UNDERSCORE: &'static TextSlice = unsafe { TextSlice::new_unchecked("_") };

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

	#[cfg(feature = "list-extensions")]
	pub fn filter(&self, block: &Value<'e>, env: &mut Environment<'e>) -> Result<Self> {
		const UNDERSCORE: &'static TextSlice = unsafe { TextSlice::new_unchecked("_") };

		let arg = env.lookup(UNDERSCORE).unwrap();

		self
			.iter()
			.filter_map(|ele| {
				arg.assign(ele.clone());

				block
					.run(env)
					.and_then(|b| b.to_boolean())
					.and_then(|a| a.then(|| Ok(ele.clone())).transpose())
					.transpose()
			})
			.collect()
	}
}

impl<'e> ToList<'e> for List<'e> {
	fn to_list(&self) -> Result<Self> {
		Ok(self.clone())
	}
}

impl ToBoolean for List<'_> {
	fn to_boolean(&self) -> Result<Boolean> {
		Ok(!self.is_empty())
	}
}

impl ToInteger for List<'_> {
	fn to_integer(&self) -> Result<Integer> {
		self.len().try_into()
	}
}

impl ToText for List<'_> {
	fn to_text(&self) -> Result<Text> {
		const NEWLINE: &TextSlice = unsafe { TextSlice::new_unchecked("\n") };

		self.join(NEWLINE)
	}
}

pub trait SliceFetch<'a, 'e> {
	type Output;
	fn get(self, list: &'a List<'e>) -> Option<Self::Output>;
}

impl<'a, 'e: 'a> SliceFetch<'a, 'e> for usize {
	type Output = &'a Value<'e>;
	fn get(self, list: &'a List<'e>) -> Option<Self::Output> {
		match list.inner()? {
			Inner::Boxed(ele) => (self == 0).then_some(ele),

			Inner::Slice(slice) => slice.get(self),
			Inner::Cons(lhs, _) if self < lhs.len() => lhs.get(self),
			Inner::Cons(lhs, rhs) => rhs.get(self - lhs.len()),

			Inner::Repeat(list, amount) if (list.len() * amount) < self => None,
			Inner::Repeat(list, amount) => list.get(self % amount),
		}
	}
}

impl<'e> SliceFetch<'_, 'e> for Range<usize> {
	type Output = List<'e>;

	fn get(self, list: &List<'e>) -> Option<Self::Output> {
		// shouldn't be the same, because it's already checked for.
		// assert_ne!(self.start, self.end);

		if list.len() < self.end || self.end < self.start {
			return None;
		}

		// FIXME: use optimizations
		Some(list.iter().skip(self.start).take(self.end - self.start).cloned().collect())
	}
}

impl<'a, 'e> IntoIterator for &'a List<'e> {
	type Item = &'a Value<'e>;
	type IntoIter = Iter<'a, 'e>;

	fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
		self.iter()
	}
}

#[derive(Clone)]
pub enum Iter<'a, 'e> {
	Empty,
	Boxed(&'a Value<'e>),
	Cons(Box<Self>, &'a List<'e>),
	Slice(std::slice::Iter<'a, Value<'e>>),
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
