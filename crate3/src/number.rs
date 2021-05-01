
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Number(i64);

impl Number {
	pub const unsafe fn new_unchecked(data: i64) -> Self {
		Self(data)
	}

	pub const fn inner(self) -> i64 {
		self.0
	}
}
