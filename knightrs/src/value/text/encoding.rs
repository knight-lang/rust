/// The encoding of [`Text`](super::Text)s and related types.
///
/// Base Knight requires implementations to support only a very limited subset of ASCII (more
/// specifically, the `\r`, `\n`, `\t` and `' '..='~'` characters). However, it's nice to be able to
/// support all of Unicode, or even just ASCII. As such, this implementation gives three different
/// encodings to use: [`Utf8`], [`Ascii`], and [`KnightEncoding].
pub trait Encoding {
	/// Returns whether `chr` is a valid character in this encoding.
	fn is_valid(chr: char) -> bool;

	/// Returns whether `chr` is a whitespace character in this encoding.
	fn is_whitespace(chr: char) -> bool;

	/// Returns whether `chr` is a numeric character in this encoding.
	fn is_numeric(chr: char) -> bool;

	/// Returns whether `chr` is a lowercase character in this encoding.
	fn is_lower(chr: char) -> bool;

	/// Returns whether `chr` is a uppercase character in this encoding.
	fn is_upper(chr: char) -> bool;
}

/// An [`Encoding`] which allows all `char`s, and uses the Unicode properties for the `is_*`
/// functions.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utf8;

impl Encoding for Utf8 {
	/// Always returns `true`, as all [`char`]s are valid.
	#[inline]
	fn is_valid(_: char) -> bool {
		true
	}

	/// Returns [`char::is_whitespace`].
	#[inline]
	fn is_whitespace(chr: char) -> bool {
		chr.is_whitespace()
	}

	/// Returns [`char::is_numeric`].
	#[inline]
	fn is_numeric(chr: char) -> bool {
		chr.is_numeric()
	}

	/// Returns [`char::is_lower`].
	#[inline]
	fn is_lower(chr: char) -> bool {
		chr.is_lowercase()
	}

	/// Returns [`char::is_upper`].
	#[inline]
	fn is_upper(chr: char) -> bool {
		chr.is_uppercase()
	}
}

/// An [`Encoding`] which only allows ASCII `char`s.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ascii;

impl Encoding for Ascii {
	/// Returns [`char::is_ascii`].
	#[inline]
	fn is_valid(chr: char) -> bool {
		chr.is_ascii()
	}

	/// Returns [`char::is_ascii_whitespace`].
	#[inline]
	fn is_whitespace(chr: char) -> bool {
		chr.is_ascii_whitespace()
	}

	/// Returns [`char::is_ascii_digit`].
	#[inline]
	fn is_numeric(chr: char) -> bool {
		chr.is_ascii_digit()
	}

	/// Returns [`char::is_ascii_lowercase`].
	#[inline]
	fn is_lower(chr: char) -> bool {
		chr.is_ascii_lowercase()
	}

	/// Returns [`char::is_ascii_uppercase`].
	#[inline]
	fn is_upper(chr: char) -> bool {
		chr.is_ascii_uppercase()
	}
}

/// An [`Encoding`] which only allows characters explicitly supported by the Knight specs.
///
/// More specifically, the allowed characters are `\r`, `\n`, `\t`, and `' '..='~'`.
///
/// This is essentially equivalent to [`Ascii`], except the [`is_valid`](KnightEncoding::is_valid)
/// and [`is_whitespace`](KnightEncoding::is_whitespace) return different values.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KnightEncoding;

impl Encoding for KnightEncoding {
	/// Returns whether `chr` is a valid Knight character. See [`KnightEncoding`] for details.
	#[inline]
	fn is_valid(chr: char) -> bool {
		matches!(chr, '\r' | '\n' | '\t' | ' '..='~')
	}

	/// Returns whether `chr` is one of `\r`, `\n`, `\t`, or ` `.
	#[inline]
	fn is_whitespace(chr: char) -> bool {
		matches!(chr, '\r' | '\n' | '\t' | ' ')
	}

	/// Returns [`char::is_ascii_digit`].
	#[inline]
	fn is_numeric(chr: char) -> bool {
		chr.is_ascii_digit()
	}

	/// Returns [`char::is_ascii_lowercase`].
	#[inline]
	fn is_lower(chr: char) -> bool {
		chr.is_ascii_lowercase()
	}

	/// Returns [`char::is_ascii_uppercase`].
	#[inline]
	fn is_upper(chr: char) -> bool {
		chr.is_ascii_uppercase()
	}
}
