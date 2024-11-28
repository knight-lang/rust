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
	pub fn from_slice(slice: &StringSlice) -> Self {
		Self(slice.into())
	}

	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn new(source: &str, opts: &Options) -> Result<Self, StringError> {
		StringSlice::new(source, opts).map(Self::from_slice)
	}
}
