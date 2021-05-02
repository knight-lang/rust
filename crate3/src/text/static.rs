use super::{TextInner, Text};

use std::mem::ManuallyDrop;

pub struct TextStatic(ManuallyDrop<TextInner>);

impl TextStatic {
	pub const unsafe fn new_static_unchecked(data: &'static str) -> Self {
		Self(ManuallyDrop::new(TextInner::Static(data)))
	}

	pub fn text(&'static self) -> Text {
		unsafe {
			Text::from_raw(&self.0 as *const _ as *const ())
		}
	}
}
