use super::KnStr;
use crate::container::RcOrRef;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KnStrRef<'a>(RcOrRef<'a, KnStr>);

impl Default for KnStrRef<'_> {
	#[inline]
	fn default() -> Self {
		<&KnStr>::default().into()
	}
}

impl<'a> From<&'a KnStr> for KnStrRef<'a> {
	#[inline]
	fn from(str: &'a KnStr) -> Self {
		Self(str.into())
	}
}

impl std::ops::Deref for KnStrRef<'_> {
	type Target = KnStr;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
