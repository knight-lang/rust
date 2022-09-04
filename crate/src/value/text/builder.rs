use super::{SharedText, Text};

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

	pub fn push(&mut self, text: &Text) {
		self.0.push_str(text);
	}

	pub fn finish(self) -> SharedText {
		self.0.try_into().unwrap_or_else(|_| unsafe { std::hint::unreachable_unchecked() })
	}
}
