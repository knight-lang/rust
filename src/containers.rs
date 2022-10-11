use std::ops::{Deref, DerefMut};

cfg_if! {
	if #[cfg(feature = "multithreaded")] {
		pub trait MaybeSendSync: Send + Sync {}
		impl<T: Send + Sync> MaybeSendSync for T {}
	} else {
		pub trait MaybeSendSync {}
		impl<T> MaybeSendSync for T {}
	}
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefCount<T: ?Sized>(
	#[cfg(feature = "multithreaded")] std::sync::Arc<T>,
	#[cfg(not(feature = "multithreaded"))] std::rc::Rc<T>,
);
#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(RefCount<()>: Send, Sync);

impl<T: ?Sized> RefCount<T> {
	pub fn ptr_eq(&self, rhs: &Self) -> bool {
		#[cfg(feature = "multithreaded")]
		{
			std::sync::Arc::ptr_eq(&self.0, &rhs.0)
		}
		#[cfg(not(feature = "multithreaded"))]
		{
			std::rc::Rc::ptr_eq(&self.0, &rhs.0)
		}
	}
}

impl<T: ?Sized> Clone for RefCount<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<T> From<T> for RefCount<T> {
	fn from(inp: T) -> Self {
		Self(inp.into())
	}
}

impl<T: ?Sized> From<Box<T>> for RefCount<T> {
	fn from(inp: Box<T>) -> Self {
		Self(inp.into())
	}
}

impl<T: ?Sized> Deref for RefCount<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Debug)]
pub struct Mutable<T>(
	#[cfg(feature = "multithreaded")] std::sync::RwLock<T>,
	#[cfg(not(feature = "multithreaded"))] std::cell::RefCell<T>,
);
#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Mutable<()>: Send, Sync);

impl<T> From<T> for Mutable<T> {
	fn from(inp: T) -> Self {
		Self(inp.into())
	}
}

impl<T> Mutable<T> {
	pub fn read(&self) -> impl Deref<Target = T> + '_ {
		#[cfg(feature = "multithreaded")]
		{
			self.0.read().unwrap()
		}

		#[cfg(not(feature = "multithreaded"))]
		{
			self.0.borrow()
		}
	}

	pub fn write(&self) -> impl DerefMut<Target = T> + '_ {
		#[cfg(feature = "multithreaded")]
		{
			self.0.write().unwrap()
		}

		#[cfg(not(feature = "multithreaded"))]
		{
			self.0.borrow_mut()
		}
	}
}
