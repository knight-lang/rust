use crate::Value;
use std::num::NonZeroU64;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct Variable<'env>(NonZeroU64, PhantomData<&'env ()>);

struct VariableInner<'env> {
	name: String,
	value: Option<Value<'env>>
}

impl<'env> Variable<'env> {
	const fn inner_ptr(&self) -> *const VariableInner<'env> {
		self.0.get() as *const VariableInner
	}

	fn inner(&self) -> &'env VariableInner<'env> {
		unsafe {
			&*self.inner_ptr()
		}
	}

	pub fn into_raw(self) -> NonZeroU64 {
		self.0 as NonZeroU64
	}

	pub unsafe fn from_raw(raw: NonZeroU64) -> Self {
		Self(raw, PhantomData)
	}
}