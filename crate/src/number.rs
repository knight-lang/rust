use crate::value::TAG_SHIFT;
use crate::text::{ToText, Text, TextCow};
use crate::boolean::{ToBoolean, Boolean};

cfg_if! {
	if #[cfg(feature="strict-numbers")] {
		/// The number type within Knight.
		pub type NumberType = i32;
		pub type UNumberType = u32;
	} else {
		/// The number type within Knight.
		pub type NumberType = i64;
		pub type UNumberType = u64;
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Number(NumberType);

pub trait ToNumber {
	fn to_number(&self) -> crate::Result<Number>;
}

const fn truncate(num: NumberType) -> NumberType {
	(num << TAG_SHIFT) >> TAG_SHIFT
}

const_assert!(Number::new(Number::MAX.get()).is_some());
const_assert!(Number::new(Number::MIN.get()).is_some());
const_assert!(Number::new(Number::MAX.get() + 1).is_none());
const_assert!(Number::new(Number::MIN.get() - 1).is_none());

impl Number {
	pub const MAX: Number = Self(((UNumberType::MAX) >> (TAG_SHIFT + 1)) as NumberType);
	pub const MIN: Number = Self(!Self::MAX.0);

	pub const fn is_valid(num: NumberType) -> bool {
		truncate(num) == num
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

	pub const fn new_truncate(num: NumberType) -> Self {
		Self(truncate(num))
	}

	#[deprecated]
	pub const fn inner(self) -> NumberType {
		self.0
	}

	pub const fn get(self) -> NumberType {
		self.0
	}
}

impl ToNumber for Number {
	fn to_number(&self) -> crate::Result<Number> {
		Ok(*self)
	}
}

impl ToBoolean for Number {
	fn to_boolean(&self) -> crate::Result<Boolean> {
		Ok(self.get() != 0)
	}
}

impl ToText for Number {
	fn to_text(&self) -> crate::Result<TextCow> {
		todo!();
	}
}