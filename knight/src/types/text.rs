mod r#static;
mod r#ref;
mod owned;
mod inner;
mod builder;
use inner::TextInner;
pub use r#static::TextStatic;
pub use r#ref::TextRef;
pub use owned::{Text, NumberOverflow};
pub use builder::TextBuilder;


#[derive(Debug, PartialEq, Eq)]
pub struct InvalidText {
	pub index: usize,
	pub byte: u8
}

#[cfg_attr(not(feature="disallow-unicode"), inline)]
pub fn validate_text(text: &str) -> Result<(), InvalidText> {
	#[cfg(feature="disallow-unicode")]
	for (index, byte) in text.bytes().enumerate() {
		if !matches!(byte, b'\r' | b'\n' | b'\t' | b' '..=b'~') {
			return Err(InvalidText { index, byte })
		}
	}

	// note that for `not(feature="disallow-unicode")`, all text is valid.
	Ok(())
}
