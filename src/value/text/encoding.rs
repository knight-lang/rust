use super::{Character, NewTextError};

fn validate_every_character<E: Encoding>(s: &str) -> Result<(), NewTextError> {
	// We're in const context, so we must use `while` with bytes.
	// Since we're not using unicode, everything's just a byte anyways.
	for (index, chr) in s.char_indices() {
		if Character::<E>::new(chr).is_none() {
			// Since everything's a byte, the byte index is the same as the char index.
			return Err(NewTextError::IllegalChar { chr, index });
		}
	}

	Ok(())
}

pub trait Encoding:
	std::fmt::Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash
{
	fn is_valid(chr: char) -> bool;
	fn is_whitespace(chr: char) -> bool;
	fn is_numeric(chr: char) -> bool;
	fn is_lowercase(chr: char) -> bool;
	fn is_uppercase(chr: char) -> bool;

	fn validate_contents(s: &str) -> Result<(), NewTextError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unicode;

impl Encoding for Unicode {
	#[inline]
	fn is_valid(_: char) -> bool {
		true
	}

	#[inline]
	fn validate_contents(_: &str) -> Result<(), super::NewTextError> {
		Ok(())
	}

	#[inline]
	fn is_whitespace(chr: char) -> bool {
		chr.is_whitespace()
	}

	#[inline]
	fn is_numeric(chr: char) -> bool {
		chr.is_numeric()
	}

	#[inline]
	fn is_lowercase(chr: char) -> bool {
		chr.is_lowercase()
	}

	#[inline]
	fn is_uppercase(chr: char) -> bool {
		chr.is_uppercase()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KnightEncoding;

impl Encoding for KnightEncoding {
	#[inline]
	fn is_valid(chr: char) -> bool {
		matches!(chr, '\r' | '\n' | '\t' | ' '..='~')
	}

	fn validate_contents(s: &str) -> Result<(), super::NewTextError> {
		validate_every_character::<Self>(s)
	}

	#[inline]
	fn is_whitespace(chr: char) -> bool {
		chr.is_ascii_whitespace()
	}

	#[inline]
	fn is_numeric(chr: char) -> bool {
		chr.is_ascii_digit()
	}

	#[inline]
	fn is_lowercase(chr: char) -> bool {
		chr.is_ascii_lowercase()
	}

	#[inline]
	fn is_uppercase(chr: char) -> bool {
		chr.is_ascii_uppercase()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ascii;

impl Encoding for Ascii {
	#[inline]
	fn is_valid(chr: char) -> bool {
		chr.is_ascii()
	}

	fn validate_contents(s: &str) -> Result<(), super::NewTextError> {
		if s.is_ascii() {
			return Ok(());
		}

		Err(validate_every_character::<Self>(s).unwrap_err())
	}

	#[inline]
	fn is_whitespace(chr: char) -> bool {
		chr.is_ascii_whitespace()
	}

	#[inline]
	fn is_numeric(chr: char) -> bool {
		chr.is_ascii_digit()
	}

	#[inline]
	fn is_lowercase(chr: char) -> bool {
		chr.is_ascii_lowercase()
	}

	#[inline]
	fn is_uppercase(chr: char) -> bool {
		chr.is_ascii_uppercase()
	}
}
