use crate::Function;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::value::{Value, Tag, ValueKind};
use crate::ops::Runnable;
use std::{borrow::Borrow, ops::Deref};
use std::fmt::{self, Debug, Formatter};
use std::ptr::NonNull;
use std::mem::{self, ManuallyDrop};
use std::alloc::{dealloc, Layout};

mod builder;
pub use builder::AstBuilder;

#[repr(transparent)]
pub struct Ast<'env>(NonNull<AstInner<'env>>);

#[repr(C, align(8))]
struct AstInner<'env> {
	// note rc needs to be at the start to align up with `Text` and `Custom`
	rc: AtomicUsize,
	func: &'env Function,
	_args: [Value<'env>; 1] // just a placeholder, we may have more than one arg, or zero.
}

impl Clone for Ast<'_> {
	fn clone(&self) -> Self {
		self.inner().rc.fetch_add(1, Ordering::Acquire);

		Self(self.0)
	}
}

impl Drop for Ast<'_> {
	fn drop(&mut self) {
		let rc = self.inner().rc.fetch_sub(1, Ordering::Release);

		debug_assert_ne!(rc, 0);

		if rc == 1 {
			unsafe {
				Self::drop_in_place(self.0.as_ptr() as *mut _)
			}
		}
	}
}

impl Debug for Ast<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct("Ast")
			.field("func", &self.inner().func)
			.field("args", &self.args())
			.finish()
	}
}

fn layout_for(arity: usize) -> Layout {
	use std::mem::{size_of, align_of};

	const BASE_SIZE: usize = size_of::<AstInner<'_>>() - size_of::<[Value<'_>; 1]>();

	Layout::from_size_align(BASE_SIZE + size_of::<Value<'_>>() * arity, align_of::<AstInner<'_>>()).unwrap()
}

impl<'env> AstInner<'env> {
	fn args_ptr_mut(&mut self) -> *mut Value<'env> {
		std::ptr::addr_of_mut!(self._args) as *mut Value<'env>
	}

	fn args_ptr(&self) -> *const Value<'env> {
		std::ptr::addr_of!(self._args) as *const Value<'env>
	}
}

impl<'env> Ast<'env> {
	pub fn new(func: &'env Function, args: impl IntoIterator<Item=Value<'env>>) -> Self {
		assert_eq!(func.arity(), args.len());

		let mut builder = Self::alloc(func);

		for value in args.into_iter() {
			builder.append(value);
		}

		let x = builder.build();
		x
	}

	pub fn alloc(func: &'env Function) -> AstBuilder<'env> {
		AstBuilder::new(func)
	}

	fn inner(&self) -> &AstInner<'env> {
		unsafe {
			&*self.0.as_ptr()
		}
	}

	fn into_raw(self) -> *const () {
		ManuallyDrop::new(self).0.as_ptr() as _
	}

	pub(crate) unsafe fn drop_in_place(ptr: *mut ()) {
		let ptr = ptr as *mut AstInner;

		debug_assert_eq!((*ptr).rc.load(Ordering::Relaxed), 0);

		let arity = (*ptr).func.arity();

		for i in 0..arity {
			(*ptr).args_ptr_mut().offset(i as isize).drop_in_place();
		}

		dealloc(ptr as *mut u8, layout_for(arity))
	}

	pub fn func(&self) -> &'env Function {
		self.inner().func
	}

	pub fn args(&self) -> &[Value<'env>] {
		unsafe {
			std::slice::from_raw_parts((*self.0.as_ptr()).args_ptr(), self.func().arity())
		}
	}
}

impl<'env> From<Ast<'env>> for Value<'env> {
	fn from(text: Ast<'env>) -> Self {
		unsafe {
			Self::new_tagged(text.into_raw() as _, Tag::Ast)
		}
	}
}

pub struct AstRef<'a, 'env>(&'a AstInner<'env>);

impl<'env> Borrow<Ast<'env>> for AstRef<'_, 'env> {
	fn borrow(&self) -> &Ast<'env> {
		// SAFETY:
		// `Ast` is a transparent pointer to `AstInner` whereas `AstRef` is a transparent
		// reference to the same type. Since pointers and references can be transmuted safely, this is valid.
		unsafe {
			mem::transmute::<&AstRef<'_, 'env>, &Ast<'env>>(self)
		}
	}
}

impl<'env> Deref for AstRef<'_, 'env> {
	type Target = Ast<'env>;

	fn deref(&self) -> &Self::Target {
		// SAFETY:
		// `Ast` is a transparent pointer to `AstInner` whereas `AstRef` is a transparent
		// reference to the same type. Since pointers and references can be transmuted safely, this is valid.
		unsafe {
			mem::transmute::<&AstRef<'_, 'env>, &Ast<'env>>(self)
		}
	}
}

unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Ast<'env> {
	type Ref = AstRef<'value, 'env>;

	fn is_value_a(value: &Value<'env>) -> bool {
		value.tag() == Tag::Ast
	}

	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		// std::ptr::drop_in_place(self.0 as *mut AstInner);
		AstRef(&*value.ptr::<AstInner>().as_ptr())
	}
}

impl<'env> Runnable<'env> for Ast<'env> {
	fn run(&self, env: &'env  crate::Environment) -> crate::Result<Value<'env>> {
		self.func().run(self.args(), env)
	}
}
