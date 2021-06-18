use super::{AstInner, Ast};
use crate::{Function, Value};
use std::ptr::NonNull;

pub struct AstBuilder<'env> {
	ptr: NonNull<AstInner<'env>>,
	len: usize
}

impl Drop for AstBuilder<'_> {
	fn drop(&mut self) {
		todo!()
	}
}

impl<'env> AstBuilder<'env> {
	pub fn new(func: &'env Function) -> Self {
		let layout = super::layout_for(func.arity());

		let ptr = 
			NonNull::new(unsafe { std::alloc::alloc(layout) as *mut AstInner<'_> })
				.unwrap_or_else(|| std::alloc::handle_alloc_error(layout));

		Self {
			ptr,
			len: 0
		}
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.len
	}

	#[inline]
	pub fn func(&self) -> &'env Function {
		unsafe { (self.ptr.as_ref()).func }
	}

	#[inline]
	pub fn capactiy(&self) -> usize {
		self.func().arity()
	}

	pub fn append(&mut self, value: Value<'env>) {
		assert!(self.len() < self.capactiy());

		unsafe {
			self.ptr.as_mut().args_ptr_mut().offset(self.len as isize).write(value);
			self.len += 1;
		}
	}

	pub fn build(self) -> Ast<'env> {
		assert_eq!(self.len(), self.capactiy());

		Ast(std::mem::ManuallyDrop::new(self).ptr)
	}
}