mod builder;
mod sharedtext;
mod text;

pub trait ToText {
	fn to_text(&self) -> crate::Result<Text>;
}

pub use builder::Builder;
pub use sharedtext::*;
pub use text::*;

pub struct Chars<'a>(std::str::Chars<'a>);
impl<'a> Chars<'a> {
	pub fn as_text(&self) -> &'a TextSlice {
		unsafe { TextSlice::new_unchecked(self.0.as_str()) }
	}
}

impl Iterator for Chars<'_> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

/// An error that indicates a character within a Knight string wasn't valid.
#[derive(Debug, PartialEq, Eq)]
pub struct IllegalChar {
	/// The char that was invalid.
	pub chr: char,

	/// The index of the invalid char in the given string.
	pub index: usize,
}

impl std::fmt::Display for IllegalChar {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "illegal byte {:?} found at position {}", self.chr, self.index)
	}
}

impl std::error::Error for IllegalChar {}

/// Returns whether `chr` is a character that can appear within Knight.
///
/// Normally, every character is considered valid. However, when the `disallow-unicode` feature is
/// enabled, only characters which are explicitly mentioned in the Knight spec are allowed.
#[inline]
pub const fn is_valid(chr: char) -> bool {
	if cfg!(feature = "strict-charset") {
		matches!(chr, '\r' | '\n' | '\t' | ' '..='~')
	} else {
		true
	}
}

pub const fn validate(data: &str) -> Result<(), IllegalChar> {
	// All valid `str`s are valid TextSlice is normal mode.
	if cfg!(not(feature = "strict-charset")) {
		return Ok(());
	}

	// We're in const context, so we must use `while` with bytes.
	// Since we're not using unicode, everything's just a byte anyways.
	let bytes = data.as_bytes();
	let mut index = 0;

	while index < bytes.len() {
		let chr = bytes[index] as char;

		if !is_valid(chr) {
			// Since everything's a byte, the byte index is the same as the char index.
			return Err(IllegalChar { chr, index });
		}

		index += 1;
	}

	Ok(())
}

impl super::ToBoolean for Text {
	fn to_boolean(&self) -> crate::Result<super::Boolean> {
		Ok(!self.is_empty())
	}
}

impl ToText for Text {
	fn to_text(&self) -> crate::Result<Self> {
		Ok(self.clone())
	}
}

impl super::KnightType for Text {
	const TYPENAME: &'static str = "Text";
}

impl super::ToList for Text {
	fn to_list(&self) -> crate::Result<super::List> {
		Ok(self.chars().map(|c| super::Value::from(Self::try_from(c.to_string()).unwrap())).collect())
	}
}
