pub trait Encoding: 'static + Default + Clone + PartialEq {
	fn is_valid(chr: char) -> bool;
	fn is_whitespace(chr: char) -> bool;
	fn is_numeric(chr: char) -> bool;
	fn is_lower(chr: char) -> bool;
	fn is_upper(chr: char) -> bool;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unicode;

impl Encoding for Unicode {
	fn is_valid(_: char) -> bool {
		true
	}

	fn is_whitespace(chr: char) -> bool {
		chr.is_whitespace()
	}
	fn is_numeric(chr: char) -> bool {
		chr.is_numeric()
	}
	fn is_lower(chr: char) -> bool {
		chr.is_lowercase()
	}

	fn is_upper(chr: char) -> bool {
		chr.is_uppercase()
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ascii;

impl Encoding for Ascii {
	fn is_valid(chr: char) -> bool {
		chr.is_ascii()
	}

	fn is_whitespace(chr: char) -> bool {
		chr.is_ascii_whitespace()
	}
	fn is_numeric(chr: char) -> bool {
		chr.is_ascii_digit()
	}
	fn is_lower(chr: char) -> bool {
		chr.is_ascii_lowercase()
	}

	fn is_upper(chr: char) -> bool {
		chr.is_ascii_uppercase()
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KnightEncoding;

impl Encoding for KnightEncoding {
	fn is_valid(chr: char) -> bool {
		matches!(chr, '\r' | '\n' | '\t' | ' '..='~')
	}

	fn is_whitespace(chr: char) -> bool {
		matches!(chr, '\r' | '\n' | '\t' | ' ')
	}

	fn is_numeric(chr: char) -> bool {
		chr.is_ascii_digit()
	}

	fn is_lower(chr: char) -> bool {
		chr.is_ascii_lowercase()
	}

	fn is_upper(chr: char) -> bool {
		chr.is_ascii_uppercase()
	}
}
