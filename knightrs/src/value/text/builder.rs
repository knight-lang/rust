use super::*;

/// A builder for [`Text`]s.
///
/// Since [`Text`]s are immutable, this builder allows you to create one from different sources
/// without having to recheck the encoding's validity.
#[must_use]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Builder(String);

impl Builder {
	/// Creates a new, empty [`Builder`].
	pub const fn new() -> Self {
		Self(String::new())
	}

	/// Creates a new builder with the given capacity.
	pub fn with_capacity(cap: usize) -> Self {
		Self(String::with_capacity(cap))
	}

	/// Adds the given `text` to the end of the builder.
	pub fn push(&mut self, text: &TextSlice) {
		self.0.push_str(text);
	}

	/// Adds the given `chr` to the end of the builder.
	pub fn push_char(&mut self, chr: char) {
		self.0.push(chr);
	}

	/// Finishes constructing the [`Text`] and returns it.
	///
	/// Note that there's no `finish_unchecked`. You can simply do [`Text::new_unchecked`] for that.
	///
	/// # Results
	/// If [`check_container_length`](crate::env::flags::Compliance::check_container_length) is
	/// enabled, and the resulting [`Text`] is too large, an error is returned.
	pub fn finish(self, flags: &Flags) -> Result<Text, NewTextError> {
		// SAFETY: We know that `self` is comprised of only valid `E`s because it was constructed only
		// with `E`s. Additionally, since `E: Encoding` is required for creating all `TextSlice`s and
		// `Characters` (which is how `self.0` is constructed), its contents are a valid `E`.
		unsafe { Text::new_len_unchecked(self.0, flags) }
	}
}
