pub trait Encoding:
	std::fmt::Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash
{
	fn is_valid(chr: char) -> bool;

	fn is_whitespace(chr: char) -> bool;
	fn is_numeric(chr: char) -> bool;
	fn is_lowercase(chr: char) -> bool;
	fn is_uppercase(chr: char) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unicode;

impl Encoding for Unicode {
	#[inline]
	fn is_valid(_: char) -> bool {
		true
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
