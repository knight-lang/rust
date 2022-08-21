mod knstr;
mod sharedstr;
// mod builder;

pub use knstr::*;
pub use sharedstr::*;
// pub use

pub struct Chars<'a>(std::str::Chars<'a>);
impl<'a> Chars<'a> {
	pub fn as_knstr(&self) -> &'a KnStr {
		unsafe { KnStr::new_unchecked(self.0.as_str()) }
	}
}

impl Iterator for Chars<'_> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}
