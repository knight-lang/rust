use super::*;

#[repr(transparent)]
pub struct TextStatic(TextInner);

impl TextStatic {
	pub const fn new(data: &'static str) -> Result<Self, InvalidByte> {
		if let Err(err) = validate(data.as_bytes()) {
			Err(err)
		} else {
			unsafe {
				Ok(Self::new_unchecked(data))
			}
		}
	}

	pub const unsafe fn new_unchecked(data: &'static str) -> Self {
		debug_assert_const!(validate(data.as_bytes()).is_ok());

		Self(TextInner {
			refcount: AtomicUsize::new(0), // irrelevant
			flags: Flags::NONE,
			kind: TextKind {
				heap: TextKindPointer { // not technically heap allocated, but whatever
					_padding: [0; TEXT_KIND_POINTER_PADDING_LEN],
					len: data.len(),
					ptr: data.as_ptr()
				}
			}
		})
	}

	pub const fn as_text(&'static self) -> Text {
		Text(&self.0)
	}
}