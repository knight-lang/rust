use super::{Text, TextSlice};

#[derive(Default, Debug, PartialEq, Eq)]
#[must_use]
pub struct Builder(String);

impl Builder {
	pub const fn new() -> Self {
		Self(String::new())
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self(String::with_capacity(cap))
	}

	pub fn push(&mut self, text: &TextSlice) {
		self.0.push_str(text);
	}

	pub fn finish(self, flags: &crate::env::Flags) -> Result<Text, super::NewTextError> {
		Text::new(self.0, flags)
	}
}
