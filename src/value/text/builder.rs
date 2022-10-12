use super::{Text, TextSlice};
use std::marker::PhantomData;

#[must_use]
#[derive_where(Default, Debug, PartialEq, Eq)]
pub struct Builder<E>(PhantomData<E>, String);

impl<E> Builder<E> {
	pub const fn new() -> Self {
		Self(PhantomData, String::new())
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self(PhantomData, String::with_capacity(cap))
	}

	pub fn push(&mut self, text: &TextSlice<E>) {
		self.1.push_str(text);
	}

	pub fn finish(self, flags: &crate::env::Flags) -> Result<Text<E>, super::NewTextError>
	where
		E: super::Encoding,
	{
		Text::new(self.1, flags)
	}
}
