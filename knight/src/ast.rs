use crate::Function;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::value::{Value, Tag, ValueKind};
use std::{borrow::Borrow, ops::Deref};
use std::fmt::{self, Debug, Formatter};

use std::alloc::{alloc, dealloc, Layout};

pub struct Ast(*const AstInner);

#[repr(C, align(8))]
struct AstInner {
	rc: AtomicUsize,
	func: &'static Function,
	_args: [Value; 1] // just a placeholder, we may have more than one arg, or zero.
}

impl Clone for Ast {
	fn clone(&self) -> Self {
		self.inner().rc.fetch_add(1, Ordering::Relaxed);

		Self(self.0)
	}
}

impl Drop for Ast {
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

impl Debug for Ast {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct("Ast")
			.field("func", &self.inner().func)
			.field("args", &self.args())
			.finish()
	}
}

fn layout_for(arity: usize) -> Layout {
	use std::mem::{size_of, align_of};

	const BASE_SIZE: usize = size_of::<AtomicUsize>() + size_of::<Function>() - size_of::<Value>();

	Layout::from_size_align(BASE_SIZE + size_of::<Value>() * arity, align_of::<AstInner>()).unwrap()
}

impl AstInner {
	fn args_ptr_mut(&mut self) -> *mut Value {
		std::ptr::addr_of_mut!(self._args) as *mut Value
	}

	fn args_ptr(&self) -> *const Value {
		std::ptr::addr_of!(self._args) as *const Value
	}
}

impl Ast {
	pub fn new(func: &'static Function, args: Box<[Value]>) -> Self {
		use std::mem::{self, ManuallyDrop};

		assert_eq!(func.arity(), args.len());

		let mut builder = Self::alloc(func);

		// copy over the arguments, and then build it.
		unsafe {
			let mut args = mem::ManuallyDrop::new(args);
			std::ptr::copy_nonoverlapping(args.as_mut_ptr(), builder.args_ptr(), mem::size_of::<Value>() * func.arity());

			Self(ManuallyDrop::new(builder).inner as *const AstInner)
		}
	}

	pub(crate) fn alloc(func: &'static Function) -> AstBuilder {
		AstBuilder::new(func)
	}

	fn inner(&self) -> &AstInner {
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

		(*ptr).args_ptr_mut().write(Value::default()); // overwrite it with an empty value, so it wont be freed.
		ptr.drop_in_place();
		dealloc(ptr as *mut u8, layout_for(arity))
	}

	pub fn func(&self) -> &Function {
		&self.inner().func
	}

	pub fn args(&self) -> &[Value] {
		unsafe {
			std::slice::from_raw_parts((*self.0).args_ptr(), self.func().arity())
		}
	}
}

impl From<Ast> for Value {
	fn from(text: Ast) -> Self {
		unsafe {
			Self::new_tagged(text.into_raw() as _, Tag::Ast)
		}
	}
}

pub struct AstRef<'a>(&'a AstInner);

impl Borrow<Ast> for AstRef<'_> {
	fn borrow(&self) -> &Ast {
		todo!()
	}
}

impl Deref for AstRef<'_> {
	type Target = Ast;

	fn deref(&self) -> &Self::Target {
		// SAFETY:
		// `Ast` is a transparent pointer to `AstInner` whereas `AstRef` is a transparent
		// reference to the same type. Since pointers and references can be transmuted safely, this is valid.
		unsafe {
			std::mem::transmute::<&AstRef<'_>, &Ast>(self)
		}
	}
}

unsafe impl<'a> ValueKind<'a> for Ast {
	type Ref = AstRef<'a>;

	fn is_value_a(value: &Value) -> bool {
		value.tag() == Tag::Ast
	}

	unsafe fn downcast_unchecked(value: &'a Value) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		// std::ptr::drop_in_place(self.0 as *mut AstInner);
		AstRef(&*(value.ptr() as *const AstInner))
	}

	fn run(&self) -> crate::Result<Value> {
		self.func().run(self.args())
	}
}

#[must_use="not using this will leak memory."]
#[allow(unused)]
pub(crate) struct AstBuilder {
	inner: *mut AstInner,
	next_insert_location: usize
}

// make sure we don't leak memory by never actually calling the drop attr.
#[cfg(debug_asseritons)]
impl Drop for AstBuilder {
	fn drop(&mut self) {
		panic!("memory leaked because AstBuilder was dropped?");
	}
}

#[allow(unused)]
impl AstBuilder {
	pub fn new(func: &'static Function) -> Self {
		use std::ptr;

		unsafe {
			let inner = alloc(layout_for(func.arity())) as *mut AstInner;
			ptr::write(ptr::addr_of_mut!((*inner).rc), AtomicUsize::new(1));
			ptr::write(ptr::addr_of_mut!((*inner).func), func);

			Self { inner, next_insert_location: 0 }
		}
	}

	fn args_ptr(&mut self) -> *mut Value {
		unsafe { &mut (*self.inner) }.args_ptr_mut()
	}

	pub fn arity(&self) -> usize {
		unsafe { &(*self.inner).func }.arity()
	}

	pub unsafe fn set_next(&mut self, value: Value) {
		debug_assert!(self.arity() <= self.next_insert_location);

		self.args_ptr().offset(self.next_insert_location as isize).write(value);
		self.next_insert_location += 1;
	}

	pub unsafe fn build(self) -> Ast {
		debug_assert_eq!(self.next_insert_location, self.arity());

		Ast(std::mem::ManuallyDrop::new(self).inner as *const AstInner)
	}
}
