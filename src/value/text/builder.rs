use super::{Text, TextSlice};
use std::marker::PhantomData;

#[derive(Default, Debug, PartialEq, Eq)]
#[must_use]
pub struct Builder<E>(String, PhantomData<E>);

impl<E> Builder<E> {
	pub const fn new() -> Self {
		Self(String::new(), PhantomData)
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self(String::with_capacity(cap))
	}

	pub fn push(&mut self, text: &TextSlice<E>) {
		self.0.push_str(text);
	}

	pub fn finish(self) -> Text<E> {
		self.0.try_into().unwrap_or_else(|_| unsafe { std::hint::unreachable_unchecked() })
	}
}
