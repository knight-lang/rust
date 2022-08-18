use crate::{value::Number, Error, Result};
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Text(Rc<str>);

impl Default for Text {
	fn default() -> Self {
		Text(Rc::from(""))
	}
}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl std::ops::Deref for Text {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// An error that indicates a character within a Knight string wasn't valid.
#[derive(Debug, PartialEq, Eq)]
pub struct IllegalByte {
	/// The byte that was invalid.
	pub byte: u8,
	/// The index of the invalid byte in the given string.
	pub index: usize,
}

impl Display for IllegalByte {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "illegal byte {:?} found at position {}", self.byte, self.index)
	}
}

impl std::error::Error for IllegalByte {}

pub const fn is_valid_char(chr: char) -> bool {
	!cfg!(feature = "disallow-unicode") || matches!(chr, '\r' | '\n' | '\t' | ' '..='~')
}

pub fn validate(data: &str) -> std::result::Result<(), IllegalByte> {
	if cfg!(not(feature = "disallow-unicode")) {
		return Ok(());
	}

	// We're in const context, so we must use `while` with bytes.
	// Since we're not using unicode, everything's just a byte anyways.
	let bytes = data.as_bytes();
	let mut index = 0;

	while index < bytes.len() {
		let byte = bytes[index];

		if !char::from_u32(byte as u32).map_or(false, is_valid_char) {
			return Err(IllegalByte { byte, index });
		}

		index += 1
	}

	Ok(())
}

impl Text {
	pub fn new(inp: impl ToString) -> std::result::Result<Self, IllegalByte> {
		let inp = inp.to_string();

		if let Err(err) = validate(&inp) {
			return Err(err);
		}

		Ok(Self(Rc::from(inp)))
	}

	pub fn to_number(&self) -> Result<Number> {
		todo!()
		// let mut chars = text.trim().bytes();
		// let mut sign = 1;
		// let mut number: Number = 0;

		// match chars.next() {
		// 	Some(b'-') => sign = -1,
		// 	Some(b'+') => { /* do nothing */ }
		// 	Some(digit @ b'0'..=b'9') => number = (digit - b'0') as _,
		// 	_ => return Ok(0),
		// };

		// while let Some(digit @ b'0'..=b'9') = chars.next() {
		// 	cfg_if! {
		// 		 if #[cfg(feature="checked-overflow")] {
		// 			  number = number
		// 					.checked_mul(10)
		// 					.and_then(|num| num.checked_add((digit as u8 - b'0') as _))
		// 					.ok_or_else(|| {
		// 						 let err: Error = error_inplace!(Error::TextConversionOverflow);
		// 						 err
		// 					})?;
		// 		 } else {
		// 			  number = number.wrapping_mul(10).wrapping_add((digit as u8 - b'0') as _);
		// 		 }
		// 	}
		// }

		// Ok(sign * number) // todo: check for this erroring. ?
	}
}
