use std::num::NonZeroU64;

pub struct Ast<'env> {
	x: &'env ()
}

impl<'env> Ast<'env> {
	pub fn into_raw(self) -> NonZeroU64 {
		todo!()
	}

	pub unsafe fn from_raw(raw: NonZeroU64) -> Self {
		todo!()
	}
}