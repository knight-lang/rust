use std::fmt::{self, Debug, Formatter};
use crate::value::{Value, Tag, ValueKind};
use crate::ops::Runnable;
use crate::{Environment, Error};
use std::cell::RefCell;
use std::ptr::NonNull;

/// A Variable within Knight,  which can be used to store values.
///
/// Each variable is associated with a specific [`Environment`], and may not live longer than it
// NOTE: you can copy variables as they're just references. The environment drops the `VariableInner` for us.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Variable<'env>(NonNull<VariableInner<'env>>);

struct VariableInner<'env> {
	name: Box<str>,
	value: RefCell<Option<Value<'env>>>
}

impl Debug for Variable<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("Variable")
				.field("name", &self.name())
				.field("value", &self.inner().value.borrow())
				.finish()
		} else {
			f.debug_tuple("Variable")
				.field(&self.name())
				.finish()
		}
	}
}

impl Eq for Variable<'_> {}
impl PartialEq for Variable<'_> {
	/// Variables are only equal if they come from the same [`Environment`] and have the same [`name`](Variable::name).
	fn eq(&self, rhs: &Self) -> bool {
		(self.0.as_ptr() as *const _) == (rhs.0.as_ptr() as *const _)
	}
}

impl<'env> Variable<'env> {
	pub(crate) fn new(name: Box<str>) -> Self {
		let inner = Box::new(VariableInner { name, value: RefCell::new(None) });

		Self(unsafe { NonNull::new_unchecked(Box::leak(inner)) })
	}

	fn into_raw(self) -> *const () {
		self.0.as_ptr() as _
	}

	// // SAFETY: `raw` must have been returned from `into_raw`
	// unsafe fn from_raw(raw: *const ()) -> Self {
	// 	debug_assert!(!raw.is_null());

	// 	Self(NonNull::new_unchecked(raw as *mut VariableInner<'env>))
	// }

	fn inner(self) -> &'env VariableInner<'env> {
		unsafe { &*self.0.as_ptr() }
	}

	/// Gets the name associated with this variable.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Environment;
	/// let env = Environment::default();
	/// let var = env.fetch_var("foo");
	///
	/// assert_eq!(var.name(), "foo");
	/// ```
	pub fn name(self) -> &'env str {
		&self.inner().name
	}

	/// Fetches the variable associated with this variable.
	///
	/// This returns [`None`] if this variable was never [assigned to](Self::set).
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Environment, Value, Boolean};
	/// let env = Environment::default();
	/// let var = env.fetch_var("foo");
	///
	/// // If the variable hasn't been assigned, it's `None`.
	/// assert!(var.get().is_none());
	///
	/// // after assigning it, it'll return the assigned value.
	/// var.set(Value::TRUE);
	/// assert_eq!(var.get().unwrap().downcast::<Boolean>(), Some(true));
	/// ```
	pub fn get(self) -> Option<Value<'env>> {
		self.inner().value.borrow().clone()
	}

	/// Associates `value` with `self`, returning its previously assigned value (if any).
	///
	/// Each time [`Variable::get()`] is called on `self`, it'll return `value` (up until [`set`](Self::set) is
	/// called again)
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Environment, Value, Boolean, Null};
	/// let env = Environment::default();
	/// let var = env.fetch_var("foo");
	///
	/// // If the variable hasn't been assigned before, `None` is returned.
	/// assert!(var.set(Value::NULL).is_none());
	///
	/// // If it's been assigned before, that previous value will be returned.
	/// assert!(var.set(Value::FALSE).unwrap().is_a::<Null>());
	/// ```
	pub fn set(self, value: Value<'env>) -> Option<Value<'env>> {
		self.inner().value.borrow_mut().replace(value)
	}

	// Drops the pointer in place.
	// SAFETY: Must only be called from the environment's `drop` function.
	pub(crate) unsafe fn drop_in_place(self) {
		(self.0.as_ptr() as *mut VariableInner<'env>).drop_in_place();
	}
}

impl<'env> From<Variable<'env>> for Value<'env> {
	fn from(var: Variable<'env>) -> Self {
		unsafe {
			Self::new_tagged(var.into_raw() as _, Tag::Variable)
		}
	}
}

unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Variable<'env> {
	type Ref = Self;

	fn is_value_a(value: &Value<'env>) -> bool {
		value.tag() == Tag::Variable
	}

	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		Self(value.ptr::<VariableInner>())
	}
}

impl<'env> Runnable<'env> for Variable<'env> {
	fn run(&self, _env: &'env Environment) -> crate::Result<Value<'env>> {
		self.get().ok_or_else(|| Error::UndefinedVariable(self.inner().name.clone()))
	}
}
