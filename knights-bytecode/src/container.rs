use std::ops::{Deref, DerefMut};

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
pub struct Mutable<T>(
	#[cfg(feature = "multithreaded")] std::sync::RwLock<T>,
	#[cfg(not(feature = "multithreaded"))] std::cell::RefCell<T>,
);

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
