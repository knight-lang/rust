use crate::Function;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::value::{Value, Tag, ValueKind, Runnable};
use std::{borrow::Borrow, ops::Deref};
use std::fmt::{self, Debug, Formatter};

use std::alloc::{alloc, dealloc, Layout};

pub struct Ast<'env>(*const AstInner<'env>);

#[repr(C, align(8))] // todo: why do we needthese? should rc at the start be enough
struct AstInner<'env> {
	rc: AtomicUsize,
	func: &'env Function,
	_args: [Value<'env>; 1] // just a placeholder, we may have more than one arg, or zero.
}

impl Clone for Ast<'_> {
	fn clone(&self) -> Self {
		self.inner().rc.fetch_add(1, Ordering::Relaxed);

		Self(self.0)
	}
}

impl Drop for Ast<'_> {
	fn drop(&mut self) {
		let rc = self.inner().rc.fetch_sub(1, Ordering::Relaxed);

		debug_assert_ne!(rc, 0);

		if rc == 1 {
			unsafe {
				Ast::drop_in_place(self.0 as *mut _)
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
	pub fn new(func: &'env Function, args: Box<[Value<'env>]>) -> Self {
		use std::mem::{self, ManuallyDrop};

		assert_eq!(func.arity(), args.len());

		let mut builder = Self::alloc(func);

		// copy over the arguments, and then build it.
		unsafe {
			let mut args = mem::ManuallyDrop::new(args);
			std::ptr::copy_nonoverlapping(args.as_mut_ptr(), builder.args_ptr(), mem::size_of::<Value>() * func.arity());

			Self(ManuallyDrop::new(builder).inner as *const AstInner<'env>)
		}
	}

	pub(crate) fn alloc(func: &'env Function) -> AstBuilder<'env> {
		AstBuilder::new(func)
	}

	fn inner(&self) -> &AstInner<'env> {
		unsafe {
			&*self.0
		}
	}

	fn into_raw(self) -> *const () {
		std::mem::ManuallyDrop::new(self).0 as _
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
			std::slice::from_raw_parts((*self.0).args_ptr(), self.func().arity())
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
			std::mem::transmute::<&AstRef<'_, 'env>, &Ast<'env>>(self)
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
			std::mem::transmute::<&AstRef<'_, 'env>, &Ast<'env>>(self)
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
		AstRef(&*(value.ptr() as *const AstInner))
	}
}

impl<'env> Runnable<'env> for Ast<'env> {
	fn run(&self, env: &'env  crate::Environment) -> crate::Result<Value<'env>> {
		self.func().run(self.args(), env)
	}
}

#[must_use="not using this will leak memory."]
#[allow(unused)]
pub(crate) struct AstBuilder<'env> {
	inner: *mut AstInner<'env>,
	next_insert_location: usize
}

// make sure we don't leak memory by never actually calling the drop attr.
#[cfg(debug_asseritons)]
impl Drop for AstBuilder<'_> {
	fn drop(&mut self) {
		panic!("memory leaked because AstBuilder was dropped?");
	}
}

#[allow(unused)]
impl<'env> AstBuilder<'env> {
	pub fn new(func: &'env Function) -> Self {
		use std::ptr;

		unsafe {
			let inner = alloc(layout_for(func.arity())) as *mut AstInner;
			ptr::write(ptr::addr_of_mut!((*inner).rc), AtomicUsize::new(1));
			ptr::write(ptr::addr_of_mut!((*inner).func), func);

			Self { inner, next_insert_location: 0 }
		}
	}

	fn args_ptr(&mut self) -> *mut Value<'env> {
		unsafe { &mut (*self.inner) }.args_ptr_mut()
	}

	pub fn arity(&self) -> usize {
		unsafe { &(*self.inner).func }.arity()
	}

	pub unsafe fn set_next(&mut self, value: Value<'env>) {
		debug_assert!(self.arity() <= self.next_insert_location);

		self.args_ptr().offset(self.next_insert_location as isize).write(value);
		self.next_insert_location += 1;
	}

	pub unsafe fn build(self) -> Ast<'env> {
		debug_assert_eq!(self.next_insert_location, self.arity());

		Ast(std::mem::ManuallyDrop::new(self).inner as *const AstInner<'env>)
	}
}
