use crate::Environment;
use std::borrow::Borrow;

use crate::container::RefCount;
use crate::options::Options;
use crate::strings::{StringError, StringSlice};
use crate::value::{Boolean, Integer, List, ToBoolean, ToInteger, ToList};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // TODO, debug
pub struct KString(RefCount<StringSlice>);

pub trait ToKString {
	fn to_kstring(&self, env: &mut Environment) -> crate::Result<KString>;
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
	pub fn new(
		source: impl AsRef<str>,
		opts: &Options,
	) -> Result<Self, crate::strings::StringError> {
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

impl ToBoolean for KString {
	fn to_boolean(&self, env: &mut Environment) -> crate::Result<Boolean> {
		todo!()
	}
}

impl ToKString for KString {
	fn to_kstring(&self, _: &mut Environment) -> crate::Result<KString> {
		Ok(self.clone())
	}
}

impl ToInteger for KString {
	fn to_integer(&self, env: &mut Environment) -> crate::Result<Integer> {
		todo!()
	}
}

impl ToList for KString {
	fn to_list(&self, env: &mut Environment) -> crate::Result<List> {
		todo!()
	}
}
