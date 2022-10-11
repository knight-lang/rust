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
