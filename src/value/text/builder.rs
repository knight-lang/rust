use super::*;
use std::marker::PhantomData;

/// A builder for [`Text`]s.
///
/// Since [`Text`]s are immutable, this builder allows you to create one from different sources
/// without having to recheck the encoding's validity.
#[must_use]
#[derive_where(Default, Debug, PartialEq, Eq)]
pub struct Builder<E>(String, PhantomData<E>);

impl<E> Builder<E> {
	/// Creates a new, empty [`Builder`].
	pub const fn new() -> Self {
		Self(String::new(), PhantomData)
	}

	/// Creates a new builder with the given capacity.
	pub fn with_capacity(cap: usize) -> Self {
		Self(String::with_capacity(cap), PhantomData)
	}

	/// Adds the given `text` to the end of the builder.
	pub fn push(&mut self, text: &TextSlice<E>) {
		self.0.push_str(text);
	}

	/// Adds the given `chr` to the end of the builder.
	pub fn push_char(&mut self, chr: Character<E>) {
		self.0.push(chr.inner());
	}

	/// Finishes constructing the [`Text`] and returns it.
	///
	/// Note that there's no `finish_unchecked`. You can simply do [`Text::new_unchecked`] for that.
	///
	/// # Results
	/// If [`check_container_length`](crate::env::flags::Compliance::check_container_length) is
	/// enabled, and the resulting [`Text`] is too large, an error is returned.
	pub fn finish(self, flags: &Flags) -> Result<Text<E>, NewTextError> {
		// SAFETY: We know that `self` is comprised of only valid `E`s because it was constructed only
		// with `E`s. Additionally, since `E: Encoding` is required for creating all `TextSlice`s and
		// `Characters` (which is how `self.0` is constructed), its contents are a valid `E`.
		unsafe { Text::new_len_unchecked(self.0, flags) }
	}
}
