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
