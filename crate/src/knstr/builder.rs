use super::{KnStr, SharedStr};

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

	pub fn push(&mut self, knstr: &KnStr) {
		self.0.push_str(knstr);
	}

	pub fn finish(self) -> SharedStr {
		self.0.try_into().unwrap_or_else(|_| unsafe { std::hint::unreachable_unchecked() })
	}
}
