use crate::value::TAG_BITS;
use crate::{Text, Boolean};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Number(i64);

impl Number {
	pub const ZERO: Self = Self(0);

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

	pub const fn new_truncate(data: i64) -> Self {
		Self((data << TAG_BITS) >> TAG_BITS)
	}

	pub const fn inner(self) -> i64 {
		self.0
	}
}

impl From<Number> for Boolean {
	#[inline]
	fn from(number: Number) -> Self {
		number.inner() != 0
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

impl Display for Number {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.inner(), f)
	}
}

#[derive(Debug)]
pub struct NumberTooLarge {
	pub parsed: i64
}

impl Display for NumberTooLarge {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "text contains a number that is too large")
	}
}

impl std::error::Error for NumberTooLarge {}

impl std::str::FromStr for Number {
	type Err = NumberTooLarge;

	fn from_str(input: &str) -> Result<Self, Self::Err> {
		let mut chars = input.trim().bytes();
		let mut sign = 1;
		let mut number = 0i64;

		match chars.next() {
			Some(b'-') => sign = -1,
			Some(b'+') => { /* do nothing */ },
			Some(digit @ b'0'..=b'9') => number = (digit - b'0') as i64,
			_ => return Ok(Self::ZERO)
		};

		while let Some(digit @ b'0'..=b'9') = chars.next() {
			if let Some(new) = number.checked_mul(10).and_then(|n| n.checked_add((digit - b'0') as i64)) {
				number = new;
			} else {
				return Err(NumberTooLarge { parsed: number })
			}
		}

		number
			.checked_mul(sign)
			.and_then(Self::new)
			.ok_or(NumberTooLarge { parsed: number })
	}
}
