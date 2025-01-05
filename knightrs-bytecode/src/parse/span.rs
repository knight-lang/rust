use super::Origin;
use crate::strings::KnStr;

/// A [`Span`] represents a portion of a snippet program.
#[derive(Debug, Clone, Copy, Hash)]
pub struct Span<'src> {
	/// The piece of code that this `Span` covers.
	pub snippet: &'src KnStr,

	/// Whence the code came.
	pub origin: Origin<'src>,
}

impl PartialEq for Span<'_> {
	/// Returns whether `self` and `rhs` point to the exact same source program.
	#[inline]
	fn eq(&self, rhs: &Self) -> bool {
		let equal = std::ptr::eq(self.snippet, rhs.snippet);

		debug_assert!(!equal || self.origin == rhs.origin, "snippets equal but not origin?");

		equal
	}
}

impl<'src> Span<'src> {
	/// Returns the line number corresponding to `self` via the given input string `origin`.
	///
	/// # Panics
	/// Panics if `self.snippet` didn't originate from `origin`.
	#[cfg(feature = "qol")]
	pub fn lineno_from(&self, source: &'src str) -> usize {
		let mut chars = source.chars();
		let mut lineno = 1;

		while chars.as_str().as_ptr() != self.snippet.as_str().as_ptr() {
			if '\n' == chars.next().expect("somehow `snippet` didn't equal `origin`") {
				lineno += 1
			}
		}

		lineno
	}
}
