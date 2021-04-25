// #![warn(missing_docs, missing_doc_code_examples)]
#![allow(clippy::tabs_in_doc_comments, unused_unsafe)]
#![warn(/*, missing_doc_code_examples, missing_docs*/)]

#[macro_export]
macro_rules! static_text {
	($text:literal) => {{
		static mut _KNIGHT_STATIC_TEXT: $crate::text::TextInner = $crate::text::TextInner::new_const($text);
		unsafe { _KNIGHT_STATIC_TEXT.into_text() }
	}};
}

extern crate static_assertions as sa;

pub mod function;
pub mod text;
mod value;
mod error;
mod stream;
pub mod environment;

/// The number type within Knight.
pub type Number = i64;

#[doc(inline)]
pub use text::Text;

#[doc(inline)]
pub use function::Function;

pub use stream::Stream;
pub use environment::{Environment, Variable};
pub use value::Value;
pub use error::{ParseError, RuntimeError};
