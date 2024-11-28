use crate::stringslice::StringError;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
	#[cfg_attr(not(feature = "compliance"), default)]
	Utf8,
	#[cfg_attr(feature = "compliance", default)]
	Knight,
	Ascii,
}

impl Encoding {
	pub fn validate(self, source: &str) -> Result<(), StringError> {
		match self {
			Self::Utf8 => Ok(()),
			Self::Knight => todo!(),
			Self::Ascii => todo!(),
		}
	}
}
