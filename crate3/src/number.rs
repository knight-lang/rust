use crate::value::TAG_BITS;
use crate::{Text, Boolean};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Number(i64);

impl Number {
	pub const fn new(data: i64) -> Option<Self> {
		if (data << TAG_BITS) >> TAG_BITS == data {
			Some(Self(data))
		} else {
			None
		}
	}

	pub const unsafe fn new_unchecked(data: i64) -> Self {
		debug_assert_eq_const!((data << TAG_BITS) >> TAG_BITS, data);

		Self(data)
	}

	pub const fn inner(self) -> i64 {
		self.0
	}
}

impl From<Number> for Boolean {
	#[inline]
	fn from(number: Number) -> Self {
		Self::new(number.inner() != 0)
	}
}

impl From<Number> for Text {
	#[inline]
	fn from(number: Number) -> Self {
		use crate::text::TextStatic;

		// write it out by and so we dont have to allocate if it's already cached.
		const ZERO: TextStatic = unsafe { TextStatic::new_static_unchecked("0") };
		const BUFLEN: usize = "-9223372036854775808".len(); // largest length string for i64.

		let mut num = number.inner();

		if num == 0 {
			return ZERO.text();
		}

		let mut buf = [0u8; BUFLEN];
		let mut pos = BUFLEN - 1;

		let is_negative = num < 0;
		num = num.abs();

		while num != 0 {
			buf[pos] = (num % 10) as u8;
			pos -= 1;
			num /= 10;
		}

		if is_negative {
			buf[pos] = b'-';
			pos -= 1;
		}

		let text = std::str::from_utf8(&buf[pos..]).unwrap();

		Self::new_borrowed(text).unwrap()
	}
}