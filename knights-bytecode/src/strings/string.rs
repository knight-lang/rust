use std::borrow::Borrow;

use super::{StringError, StringSlice};
use crate::container::RefCount;
use crate::options::Options;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct String(RefCount<StringSlice>);

impl Default for String {
	fn default() -> Self {
		Self::from_slice(Default::default())
	}
}

impl String {
	#[inline]
	pub fn from_slice(slice: &StringSlice) -> Self {
		let refcounted = RefCount::<str>::from(slice.as_str());
		// SAFETY: tood, but it is valid i think lol
		Self(unsafe { RefCount::from_raw(RefCount::into_raw(refcounted) as *const StringSlice) })
	}

	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn new(source: &str, opts: &Options) -> Result<Self, StringError> {
		StringSlice::new(source, opts).map(Self::from_slice)
	}
}

impl Borrow<StringSlice> for String {
	fn borrow(&self) -> &StringSlice {
		&self
	}
}

impl std::ops::Deref for String {
	type Target = StringSlice;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl AsRef<StringSlice> for String {
	fn as_ref(&self) -> &StringSlice {
		&self
	}
}
