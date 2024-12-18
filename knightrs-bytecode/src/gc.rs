use crate::value2::ALLOC_VALUE_SIZE_IN_BYTES;

#[derive(Default)]
pub struct Gc {
	allocs: Vec<*const [u8; ALLOC_VALUE_SIZE_IN_BYTES]>,
}

impl Gc {
	pub fn alloc_value(&mut self) -> *const [u8; ALLOC_VALUE_SIZE_IN_BYTES] {
		let b = Box::into_raw(Box::new([0; ALLOC_VALUE_SIZE_IN_BYTES]));
		self.allocs.push(b);
		b
	}
}

// safety: has to make sure there's no cycle. shouldn't be for any builtin types.
pub unsafe trait Mark {
	fn mark(&mut self);
}

pub unsafe trait Sweep {
	fn sweep(self);
}
