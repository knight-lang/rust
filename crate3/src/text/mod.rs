mod text;
mod r#ref;
mod r#static;

pub use text::*;
pub use r#ref::*;
pub use r#static::*;

enum TextInner {
	Static(&'static str),
	Arc(std::sync::Arc<str>)
}

#[derive(Debug)]
pub struct InvalidChar {
	pub chr: char,
	pub idx: usize
}

impl std::fmt::Display for InvalidChar {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "invalid character {:?} at index {}", self.chr, self.idx)
	}
}

impl std::error::Error for InvalidChar {}

#[cfg(feature = "unicode")]
fn validate(text: &str) -> Result<(), InvalidChar> {
	let _ = text;
	Ok(())
}

#[cfg(not(feature = "unicode"))]
fn validate(text: &str) -> Result<(), InvalidChar> {
	for (idx, chr) in text.bytes().enumerate() {
		if !matches!(chr, '\r' | '\n' | '\t' | ' '..='~') {
			return Err(InvalidChar { chr, idx })
		}
	}

	// todo
	Ok(())
}