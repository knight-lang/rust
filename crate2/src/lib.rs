extern crate static_assertions as sa;

macro_rules! debug_assert_const {
	($cond:expr) => {
		#[cfg(debug_assertions)]
		{
			let _ = [()][!$cond as usize];
		}
	}
}

mod error;
mod boolean;
mod number;
mod null;
mod value;
pub mod text;
pub mod environment;


pub use error::*;
pub use boolean::Boolean;
pub use number::Number;
pub use null::Null;
pub use text::Text;
pub use value::{Value, ValueKind};
pub use environment::{Variable, Environment};