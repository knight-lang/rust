use std::hash::{Hash, Hasher};

use crate::strings::KnStr;

cfg_if! {
if #[cfg(feature = "multithreaded")] {
	pub trait MaybeSendSync: Send + Sync {}
	impl<T: Send + Sync> MaybeSendSync for T {}
	pub type RefCount<T> = std::sync::Arc<T>;

} else {
	pub trait MaybeSendSync {}
	impl<T> MaybeSendSync for T {}
	pub type RefCount<T> = std::rc::Rc<T>;
}}

#[derive(Debug)]
pub struct RcOrRef<'a, T: ?Sized>(RcOrRefInner<'a, T>);

#[derive(Debug)]
enum RcOrRefInner<'a, T: ?Sized> {
	Ref(&'a T),
	Rc(RefCount<T>),
}

impl<T: ?Sized> Clone for RcOrRef<'_, T> {
	fn clone(&self) -> Self {
		match &self.0 {
			RcOrRefInner::Ref(r) => Self(RcOrRefInner::Ref(r)),
			RcOrRefInner::Rc(rc) => Self(RcOrRefInner::Rc(rc.clone())),
		}
	}
}

impl<'a, T: ?Sized> From<&'a T> for RcOrRef<'a, T> {
	fn from(r: &'a T) -> Self {
		Self(RcOrRefInner::Ref(r))
	}
}

impl<T> From<T> for RcOrRef<'_, T> {
	fn from(t: T) -> Self {
		Self(RcOrRefInner::Rc(t.into()))
	}
}

impl<T: ?Sized> From<RefCount<T>> for RcOrRef<'_, T> {
	fn from(t: RefCount<T>) -> Self {
		Self(RcOrRefInner::Rc(t))
	}
}

impl<T: Default> Default for RcOrRef<'_, T> {
	fn default() -> Self {
		T::default().into()
	}
}

impl<T: ?Sized> std::ops::Deref for RcOrRef<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		match &self.0 {
			RcOrRefInner::Ref(r) => r,
			RcOrRefInner::Rc(rc) => &rc,
		}
	}
}

impl<T: ?Sized> AsRef<T> for RcOrRef<'_, T> {
	fn as_ref(&self) -> &T {
		&self
	}
}

impl<T: PartialEq + ?Sized> PartialEq for RcOrRef<'_, T> {
	fn eq(&self, rhs: &Self) -> bool {
		&**self == &**rhs
	}
}

impl<T: Eq + ?Sized> Eq for RcOrRef<'_, T> {}
impl<T: PartialOrd + ?Sized> PartialOrd for RcOrRef<'_, T> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		self.as_ref().partial_cmp(&rhs)
	}
}

impl<T: Ord + ?Sized> Ord for RcOrRef<'_, T> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.as_ref().cmp(&rhs)
	}
}

impl<T: Hash + ?Sized> Hash for RcOrRef<'_, T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_ref().hash(state)
	}
}

impl<'a, T> RcOrRef<'a, T> {
	pub fn into_owned(self) -> RcOrRef<'static, T>
	where
		T: Clone,
	{
		match self.0 {
			RcOrRefInner::Ref(r) => r.clone().into(),
			RcOrRefInner::Rc(rc) => rc.clone().into(),
		}
	}
}

impl RcOrRef<'_, KnStr> {
	pub fn into_owned_a(self) -> RcOrRef<'static, KnStr> {
		match self.0 {
			RcOrRefInner::Ref(slice) => {
				let refcounted = RefCount::<str>::from(slice.as_str());
				// SAFETY: tood, but it is valid i think lol
				unsafe { RefCount::from_raw(RefCount::into_raw(refcounted) as *const KnStr) }.into()
			}
			RcOrRefInner::Rc(rc) => rc.clone().into(),
		}
	}
}
