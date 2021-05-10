use crate::value2::TAG_SHIFT;

cfg_if! {
	if #[cfg(feature="strict-numbers")] {
		/// The number type within Knight.
		pub type NumberType = i32;
	} else {
		/// The number type within Knight.
		pub type NumberType = i64;
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Number(NumberType);

impl Number {
	pub const fn is_valid(num: NumberType) -> bool {
		(num << TAG_SHIFT) >> TAG_SHIFT == num
	}

	pub const fn new(num: NumberType) -> Option<Self> {
		if Self::is_valid(num) {
			Some(Self(num))
		} else {
			None
		}
	}

	pub const unsafe fn new_unchecked(num: NumberType) -> Self {
		debug_assert_const!(Self::is_valid(num));

		Self(num)
	}

	pub const fn inner(self) -> NumberType {
		self.0
	}
}