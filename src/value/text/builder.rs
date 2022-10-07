use super::{Text, TextSlice};
use crate::Encoding;
use std::marker::PhantomData;

#[must_use]
pub struct Builder<E>(String, PhantomData<E>);

impl<E: Encoding> Default for Builder<E> {
	fn default() -> Self {
		Self::new()
	}
}

impl<E: Encoding> Builder<E> {
	pub const fn new() -> Self {
		Self(String::new(), PhantomData)
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self(String::with_capacity(cap), PhantomData)
	}

	pub fn push(&mut self, text: &TextSlice<E>) {
		self.0.push_str(text);
	}

	pub fn finish(self) -> Text<E> {
		unsafe { Text::new_unchecked(self.0) }
	}
}
