use super::{Text, InvalidChar};

pub struct TextBuilder(String);

impl TextBuilder {
	pub fn with_capacity(capacity: usize) -> Self {
		Self(String::with_capacity(capacity))
	}

	pub fn append(&mut self, data: &str) {
		self.0.push_str(data);
	}

	pub fn build(self) -> Result<Text, InvalidChar> {
		Text::new(self.0)
	}
}
