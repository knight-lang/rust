use std::sync::atomic::AtomicU8;

use crate::value2::ValueAlign;

#[derive(Default)]
pub struct Gc {
	value_inners: Vec<*mut ValueInner>,
	roots: Vec<*mut ValueInner>,
}

pub const ALLOC_VALUE_SIZE: usize = 32;

#[repr(C)]
pub struct ValueInner {
	_align: ValueAlign,
	pub flags: AtomicU8,
	pub data: [u8; ALLOC_VALUE_SIZE - std::mem::size_of::<AtomicU8>()],
}

impl Gc {
	pub fn alloc_value_inner(&mut self) -> *mut ValueInner {
		use std::alloc::{alloc, Layout};
		unsafe {
			let layout = Layout::new::<ValueInner>();
			let inner = std::alloc::alloc(layout).cast::<ValueInner>();
			if inner.is_null() {
				panic!("alloc failed");
			}
			self.value_inners.push(inner);
			inner
		}
	}

	pub fn mark_and_sweep(&mut self) {
		// for root in &mut self.roots {
		// 	root.mark();
		// }

		// for root in &mut self.roots {
		// 	root.sweep();
		// }
	}
}

// safety: has to make sure there's no cycle. shouldn't be for any builtin types.
pub unsafe trait Mark {
	fn mark(&mut self);
}

pub unsafe trait Sweep {
	fn sweep(self);
}
