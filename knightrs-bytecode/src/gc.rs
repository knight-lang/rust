use std::sync::atomic::AtomicU8;

use crate::value2::{Value, ValueAlign};

#[derive(Default)]
pub struct Gc {
	value_inners: Vec<*mut ValueInner>,
	roots: Vec<Value>,
}

pub const ALLOC_VALUE_SIZE: usize = 32;

#[repr(C)]
pub struct ValueInner {
	_align: ValueAlign,
	pub flags: AtomicU8,
	pub data: [u8; ALLOC_VALUE_SIZE - std::mem::size_of::<AtomicU8>()],
}

impl Gc {
	pub fn add_root(&mut self, root: Value) {
		self.roots.push(root);
	}

	pub fn alloc_value_inner(&mut self) -> *mut ValueInner {
		use std::alloc::{alloc, Layout};
		unsafe {
			let layout = Layout::new::<ValueInner>();
			let inner = alloc(layout).cast::<ValueInner>();
			if inner.is_null() {
				panic!("alloc failed");
			}
			self.value_inners.push(inner);
			inner
		}
	}

	pub fn free_value_inner(&mut self, ptr: *mut ValueInner) {
		use std::alloc::{dealloc, Layout};
		unsafe {
			let layout = Layout::new::<ValueInner>();
			dealloc(ptr.cast::<u8>(), layout);
		}
	}

	pub fn mark_and_sweep(&mut self) {
		assert_ne!(self.roots.len(), 0, "called mark_and_sweep during mark and sweep");

		for root in &mut self.roots {
			unsafe {
				root.mark();
			}
		}

		// TODO: we should be sweeping not from roots but for _all_ values
		let mut roots = std::mem::take(&mut self.roots);
		for root in &mut roots {
			unsafe {
				root.sweep(self);
			}
		}
		self.roots = roots;
	}

	pub unsafe fn shutdown(&mut self) {
		for root in std::mem::take(&mut self.roots) {
			unsafe {
				root.deallocate(self);
			}
		}
	}
}

// safety: has to make sure there's no cycle. shouldn't be for any builtin types.
pub unsafe trait Mark {
	// safety: should not be called by anyone other than `gc`
	unsafe fn mark(&mut self);
}

pub unsafe trait Sweep {
	// safety: should not be called by anyone other than `gc`
	unsafe fn sweep(self, gc: &mut Gc);
	unsafe fn deallocate(self, gc: &mut Gc);
}
