use crate::knightstr::{IllegalChar, KnightStr};
use crate::{value::Number, Error};
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Text(Rc<KnightStr>);

impl Default for Text {
	fn default() -> Self {
		Text::new("").unwrap()
	}
}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl std::ops::Deref for Text {
	type Target = KnightStr;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<Box<KnightStr>> for Text {
	fn from(kstr: Box<KnightStr>) -> Self {
		Self(kstr.into())
	}
}
impl TryFrom<String> for Text {
	type Error = IllegalChar;

	fn try_from(inp: String) -> Result<Self, Self::Error> {
		let boxed = Box::<KnightStr>::try_from(inp.into_boxed_str())?;

		Ok(Self(boxed.into()))
	}
}

impl Text {
	pub fn new(inp: impl ToString) -> Result<Self, IllegalChar> {
		inp.to_string().try_into()
	}

	pub fn to_number(&self) -> crate::Result<Number> {
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
