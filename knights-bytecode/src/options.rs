use crate::strings::Encoding;

#[derive(Default)]
pub struct Options {
	pub check_length: bool,
	pub encoding: Encoding,
}
