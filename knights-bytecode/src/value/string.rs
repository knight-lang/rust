use crate::Environment;
use std::borrow::Borrow;

use crate::container::RefCount;
use crate::options::Options;
use crate::strings::{Error, StringSlice};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KString(RefCount<StringSlice>);

pub trait ToString {
	fn to_string(&self, env: &mut Environment) -> Result<KString, crate::Error>;
}

impl Default for KString {
	fn default() -> Self {
		Self::from_slice(Default::default())
	}
}

impl KString {
	#[inline]
	pub fn from_slice(slice: &StringSlice) -> Self {
		let refcounted = RefCount::<str>::from(slice.as_str());
		// SAFETY: tood, but it is valid i think lol
		Self(unsafe { RefCount::from_raw(RefCount::into_raw(refcounted) as *const StringSlice) })
	}

	pub fn new_unvalidated(source: &str) -> Self {
		Self::from_slice(StringSlice::new_unvalidated(source))
	}

	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn new(source: impl AsRef<str>, opts: &Options) -> Result<Self, Error> {
		StringSlice::new(source, opts).map(Self::from_slice)
	}
}

impl Borrow<StringSlice> for KString {
	fn borrow(&self) -> &StringSlice {
		&self
	}
}

impl std::ops::Deref for KString {
	type Target = StringSlice;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl AsRef<StringSlice> for KString {
	fn as_ref(&self) -> &StringSlice {
		&self
	}
}
