use crate::containers::{MaybeSendSync, RefCount};
use crate::value::{
	Boolean, Integer, List, NamedType, Runnable, Text, ToBoolean, ToInteger, ToList, ToText,
};
use crate::{Environment, Error, Result, Value};
use std::cmp::Ordering;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

/// A type that can hold custom data that's not a part of vanilla Knight.
///
/// This is a simple wrapper around a [`Refcount`] of [`CustomType`]. All the meat is within
/// [`CustomType`].
#[derive(Debug, Clone)]
pub struct Custom(RefCount<dyn CustomType>);

impl<I: Eq, E> Eq for Custom {}
impl<I: PartialEq, E> PartialEq for Custom {
	fn eq(&self, rhs: &Self) -> bool {
		RefCount::ptr_eq(&self.0, &rhs.0)
	}
}

impl<I: Hash, E> Hash for Custom {
	fn hash<H: Hasher>(&self, state: &mut H) {
		(RefCount::as_ptr(&self.0) as *const u8 as usize).hash(state);
	}
}

impl<I, E, T: CustomType + 'static> From<RefCount<T>> for Custom {
	fn from(inp: RefCount<T>) -> Self {
		Self(inp as _)
	}
}

impl Custom {
	/// A helper method to create a [`Custom`].
	pub fn new<T: CustomType + 'static>(data: T) -> Self {
		Self(RefCount::from(data) as _)
	}
}

/// Trait for custom types.
///
/// The only required function is [`CustomType::to_custom`] (see below for details). Every other
/// function is supplied with a sane default (generally just returning an [`Error::NoConversion`]/
/// [`Error::TypeError`]), but can be overridden to provide actual implementations.
///
/// # `to_custom`
/// Due to limitations in the Rust's type system, there's no way to convert from an `Arc<MyType>` to
/// an `Arc<dyn CustomType>`. As such, it's required for implementations to supply this, however
/// it's literally as simple as calling `self.into()`.
///
/// # Examples
/// Here's an example implementation of a map type that would be usable within Knight.
/// ```
/// use std::collections::HashMap;
/// use std::fmt::{self, Debug, Formatter};
/// use knightrs::{RefCount, Result, Error, Environment};
/// use knightrs::value::{
/// 	integer::IntType,
/// 	text::Encoding,
/// 	custom::{Custom, CustomType},
/// 	Value,
/// };
///
/// // Our map type. In line with Knight tradition, we'll be keeping it immutable.
/// #[derive(Debug)]
/// pub struct Map(RefCount<HashMap<Value, Value>>);
///
/// // Here we're implementing the custom type trait for our map.
/// // Technically we requiring `I` to be `IntType` and `E` to be `Encoding`
/// // is a stronger requirement than we need, but for the sake of example,
/// // let's just roll with it.
/// impl<I: IntType, E: Encoding> CustomType for Map {
///    // The required function for all implementations.
///    fn to_custom(self: RefCount<Self>) -> Custom {
///       self.into()
///    }
///
///    // The length of a map is how many elements it has.
///    fn length(self: RefCount<Self>, _env: &mut Environment<'_>) -> Result<usize> {
///       Ok(self.0.len())
///    }
///
///    // We'll just ignore the length parameter given to us, and use the start
///    // as the key to index with.
///    fn get(
///       self: RefCount<Self>,
///       start: &Value,
///       _len: &Value,
///       _env: &mut Environment<'_>,
///    ) -> Result<Value> {
///       self.0
///          .get(start)
///          .ok_or_else(|| Error::Custom(format!("unknown key: {start:?}").into()))
///          .cloned()
///    }
///
///    // Like `get`, we'll be ignoring `len`. We'll be using `replacement`
///    // as the value to fetch.
///    fn set(
///       self: RefCount<Self>,
///       start: &Value,
///       _len: &Value,
///       replacement: Value,
///       _env: &mut Environment<'_>,
///    ) -> Result<Value> {
///       let mut new = (*self.0).clone();
///       new.insert(start.clone(), replacement);
///       Ok(Custom::new(Self(new.into())).into())
///    }
/// }
/// ```
#[allow(unused_variables)]
pub trait CustomType: std::fmt::Debug + MaybeSendSync {
	/// <todo>
	fn to_custom(self: RefCount<Self>) -> Custom;

	/// Returns the name of this type
	fn typename(&self) -> &'static str {
		std::any::type_name::<Self>()
	}

	fn run(self: RefCount<Self>, env: &mut Environment) -> Result<Value> {
		Ok(self.to_custom().into())
	}

	fn to_text(self: RefCount<Self>, env: &mut Environment) -> Result<Text>
	where
		I: Display,
	{
		Err(Error::NoConversion { to: Text::TYPENAME, from: self.typename() })
	}

	fn to_integer(self: RefCount<Self>, env: &mut Environment) -> Result<Integer> {
		Err(Error::NoConversion { to: Integer::TYPENAME, from: self.typename() })
	}

	fn to_boolean(self: RefCount<Self>, env: &mut Environment) -> Result<Boolean> {
		Err(Error::NoConversion { to: Boolean::TYPENAME, from: self.typename() })
	}

	fn to_list(self: RefCount<Self>, env: &mut Environment) -> Result<List> {
		Err(Error::NoConversion { to: List::TYPENAME, from: self.typename() })
	}

	fn head(self: RefCount<Self>, env: &mut Environment) -> Result<Value> {
		Err(Error::TypeError(self.typename(), "["))
	}

	fn tail(self: RefCount<Self>, env: &mut Environment) -> Result<Value> {
		Err(Error::TypeError(self.typename(), "]"))
	}

	fn length(self: RefCount<Self>, env: &mut Environment) -> Result<usize> {
		Ok(self.to_list(env)?.len())
	}

	fn ascii(self: RefCount<Self>, env: &mut Environment) -> Result<Value> {
		Err(Error::TypeError(self.typename(), "ASCII"))
	}

	fn add(self: RefCount<Self>, rhs: &Value, env: &mut Environment) -> Result<Value> {
		Err(Error::TypeError(self.typename(), "+"))
	}

	fn subtract(self: RefCount<Self>, rhs: &Value, env: &mut Environment) -> Result<Value> {
		Err(Error::TypeError(self.typename(), "-"))
	}

	fn multiply(self: RefCount<Self>, rhs: &Value, env: &mut Environment) -> Result<Value> {
		Err(Error::TypeError(self.typename(), "*"))
	}

	fn divide(self: RefCount<Self>, rhs: &Value, env: &mut Environment) -> Result<Value> {
		Err(Error::TypeError(self.typename(), "/"))
	}

	fn remainder(self: RefCount<Self>, rhs: &Value, env: &mut Environment) -> Result<Value> {
		Err(Error::TypeError(self.typename(), "%"))
	}

	fn power(self: RefCount<Self>, rhs: &Value, env: &mut Environment) -> Result<Value> {
		Err(Error::TypeError(self.typename(), "^"))
	}

	fn compare(self: RefCount<Self>, rhs: &Value, env: &mut Environment) -> Result<Ordering> {
		Err(Error::TypeError(self.typename(), "<cmp>"))
	}

	fn assign(self: RefCount<Self>, rhs: Value, env: &mut Environment) -> Result<()> {
		Err(Error::TypeError(self.typename(), "="))
	}

	fn get(
		self: RefCount<Self>,
		start: &Value,
		len: &Value,
		env: &mut Environment,
	) -> Result<Value> {
		Err(Error::TypeError(self.typename(), "GET"))
	}

	fn set(
		self: RefCount<Self>,
		start: &Value,
		len: &Value,
		replacement: Value,
		env: &mut Environment,
	) -> Result<Value> {
		Err(Error::TypeError(self.typename(), "GET"))
	}
}

impl<I: Display, E: Encoding> ToText for Custom {
	fn to_text(&self, env: &mut Environment) -> Result<Text> {
		self.0.clone().to_text(env)
	}
}

impl ToInteger for Custom {
	fn to_integer(&self, env: &mut Environment) -> Result<Integer> {
		self.0.clone().to_integer(env)
	}
}

impl ToBoolean for Custom {
	fn to_boolean(&self, env: &mut Environment) -> Result<Boolean> {
		self.0.clone().to_boolean(env)
	}
}

impl ToList for Custom {
	fn to_list(&self, env: &mut Environment) -> Result<List> {
		self.0.clone().to_list(env)
	}
}

impl Runnable for Custom {
	fn run(&self, env: &mut Environment) -> Result<Value> {
		self.0.clone().run(env)
	}
}

impl Custom {
	pub fn typename(&self) -> &'static str {
		self.0.typename()
	}

	pub fn run(&self, env: &mut Environment) -> Result<Value> {
		self.0.clone().run(env)
	}

	pub fn head(&self, env: &mut Environment) -> Result<Value> {
		self.0.clone().head(env)
	}

	pub fn tail(&self, env: &mut Environment) -> Result<Value> {
		self.0.clone().tail(env)
	}

	pub fn length(&self, env: &mut Environment) -> Result<usize> {
		self.0.clone().length(env)
	}

	pub fn ascii(&self, env: &mut Environment) -> Result<Value> {
		self.0.clone().ascii(env)
	}

	pub fn add(&self, rhs: &Value, env: &mut Environment) -> Result<Value> {
		self.0.clone().add(rhs, env)
	}

	pub fn subtract(&self, rhs: &Value, env: &mut Environment) -> Result<Value> {
		self.0.clone().subtract(rhs, env)
	}

	pub fn multiply(&self, rhs: &Value, env: &mut Environment) -> Result<Value> {
		self.0.clone().multiply(rhs, env)
	}

	pub fn divide(&self, rhs: &Value, env: &mut Environment) -> Result<Value> {
		self.0.clone().divide(rhs, env)
	}

	pub fn remainder(&self, rhs: &Value, env: &mut Environment) -> Result<Value> {
		self.0.clone().remainder(rhs, env)
	}

	pub fn power(&self, rhs: &Value, env: &mut Environment) -> Result<Value> {
		self.0.clone().power(rhs, env)
	}

	pub fn compare(&self, rhs: &Value, env: &mut Environment) -> Result<Ordering> {
		self.0.clone().compare(rhs, env)
	}

	pub fn assign(&self, rhs: Value, env: &mut Environment) -> Result<()> {
		self.0.clone().assign(rhs, env)
	}

	pub fn get(&self, start: &Value, len: &Value, env: &mut Environment) -> Result<Value> {
		self.0.clone().get(start, len, env)
	}

	pub fn set(
		&self,
		start: &Value,
		len: &Value,
		replacement: Value,
		env: &mut Environment,
	) -> Result<Value> {
		self.0.clone().set(start, len, replacement, env)
	}
}
