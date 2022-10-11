use crate::value::{
	Boolean, Integer, List, NamedType, Runnable, Text, ToBoolean, ToInteger, ToList, ToText,
};
use crate::{containers::MaybeSendSync, Environment, Error, RefCount, Result, Value};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};

pub struct Custom<'a, I, E>(RefCount<dyn CustomType<'a, I, E>>);

impl<I, E> Clone for Custom<'_, I, E> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<I: Debug, E> Debug for Custom<'_, I, E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl<I, E> PartialEq for Custom<'_, I, E> {
	fn eq(&self, rhs: &Self) -> bool {
		self.0.eq(&*rhs.0)
	}
}

pub trait CustomType<'e, I, E>: std::fmt::Debug + MaybeSendSync {
	fn typename(&self) -> &'static str {
		std::any::type_name::<Self>()
	}

	#[allow(clippy::should_implement_trait)]
	fn eq(&self, rhs: &dyn CustomType<'e, I, E>) -> bool {
		// std::ptr::eq(self as *const u8, rhs as *const u8)
		let _ = rhs;
		todo!();
	}

	fn run(
		&self,
		this: &Custom<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = env;
		Ok(this.clone().into())
	}

	fn to_text(&self, this: &Custom<'e, I, E>, env: &mut Environment<'e, I, E>) -> Result<Text<E>> {
		let _ = (this, env);
		Err(Error::NoConversion { to: Text::<E>::TYPENAME, from: self.typename() })
	}

	fn to_integer(
		&self,
		this: &Custom<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Integer<I>> {
		let _ = (this, env);
		Err(Error::NoConversion { to: Integer::<I>::TYPENAME, from: self.typename() })
	}

	fn to_boolean(
		&self,
		this: &Custom<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Boolean> {
		let _ = (this, env);
		Err(Error::NoConversion { to: Boolean::TYPENAME, from: self.typename() })
	}

	fn to_list(
		&self,
		this: &Custom<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<List<'e, I, E>> {
		let _ = (this, env);
		Err(Error::NoConversion { to: List::<I, E>::TYPENAME, from: self.typename() })
	}

	fn head(
		&self,
		this: &Custom<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = (this, env);
		Err(Error::TypeError(self.typename(), "["))
	}

	fn tail(
		&self,
		this: &Custom<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = (this, env);
		Err(Error::TypeError(self.typename(), "]"))
	}

	fn length(
		&self,
		this: &Custom<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>>
	where
		I: crate::value::integer::IntType,
	{
		Integer::<I>::try_from(self.to_list(this, env)?.len()).map(Value::from)
	}

	fn ascii(
		&self,
		this: &Custom<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = (this, env);
		Err(Error::TypeError(self.typename(), "ASCII"))
	}

	fn add(
		&self,
		this: &Custom<'e, I, E>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "+"))
	}

	fn subtract(
		&self,
		this: &Custom<'e, I, E>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "-"))
	}

	fn multiply(
		&self,
		this: &Custom<'e, I, E>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "*"))
	}

	fn divide(
		&self,
		this: &Custom<'e, I, E>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "/"))
	}

	fn remainder(
		&self,
		this: &Custom<'e, I, E>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "%"))
	}

	fn power(
		&self,
		this: &Custom<'e, I, E>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "^"))
	}

	fn compare(
		&self,
		this: &Custom<'e, I, E>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Ordering> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "<cmp>"))
	}

	fn assign(
		&self,
		this: &Custom<'e, I, E>,
		rhs: Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<()> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "ASSIGN"))
	}

	fn get(
		&self,
		this: &Custom<'e, I, E>,
		start: usize,
		len: usize,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = (this, start, len, env);
		Err(Error::TypeError(self.typename(), "GET"))
	}

	fn set(
		&self,
		this: &Custom<'e, I, E>,
		start: usize,
		len: usize,
		replacement: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		let _ = (this, start, len, replacement, env);
		Err(Error::TypeError(self.typename(), "GET"))
	}
}

impl<'e, I, E> ToText<'e, I, E> for Custom<'e, I, E> {
	fn to_text(&self, env: &mut Environment<'e, I, E>) -> Result<Text<E>> {
		self.0.to_text(self, env)
	}
}

impl<'e, I: crate::value::integer::IntType, E> ToInteger<'e, I, E> for Custom<'e, I, E> {
	fn to_integer(&self, env: &mut Environment<'e, I, E>) -> Result<Integer<I>> {
		self.0.to_integer(self, env)
	}
}

impl<'e, I, E> ToBoolean<'e, I, E> for Custom<'e, I, E> {
	fn to_boolean(&self, env: &mut Environment<'e, I, E>) -> Result<Boolean> {
		self.0.to_boolean(self, env)
	}
}

impl<'e, I, E> ToList<'e, I, E> for Custom<'e, I, E> {
	fn to_list(&self, env: &mut Environment<'e, I, E>) -> Result<List<'e, I, E>> {
		self.0.to_list(self, env)
	}
}

impl<'e, I, E> Runnable<'e, I, E> for Custom<'e, I, E> {
	fn run(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>> {
		self.0.run(self, env)
	}
}

impl<'e, I, E> Custom<'e, I, E> {
	pub fn new<T: CustomType<'e, I, E> + 'static>(data: T) -> Self {
		Self((Box::new(data) as Box<dyn CustomType<'e, I, E>>).into())
	}

	pub fn typename(&self) -> &'static str {
		self.0.typename()
	}

	pub fn run(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>> {
		self.0.run(self, env)
	}

	pub fn head(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>> {
		self.0.head(self, env)
	}

	pub fn tail(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>> {
		self.0.tail(self, env)
	}

	pub fn length(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>>
	where
		I: crate::value::integer::IntType,
	{
		self.0.length(self, env)
	}

	pub fn ascii(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>> {
		self.0.ascii(self, env)
	}

	pub fn add(
		&self,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		self.0.add(self, rhs, env)
	}

	pub fn subtract(
		&self,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		self.0.subtract(self, rhs, env)
	}

	pub fn multiply(
		&self,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		self.0.multiply(self, rhs, env)
	}

	pub fn divide(
		&self,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		self.0.divide(self, rhs, env)
	}

	pub fn remainder(
		&self,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		self.0.remainder(self, rhs, env)
	}

	pub fn power(
		&self,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		self.0.power(self, rhs, env)
	}

	pub fn compare(
		&self,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Ordering> {
		self.0.compare(self, rhs, env)
	}

	pub fn assign(&self, rhs: Value<'e, I, E>, env: &mut Environment<'e, I, E>) -> Result<()> {
		self.0.assign(self, rhs, env)
	}

	pub fn get(
		&self,
		start: usize,
		len: usize,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		self.0.get(self, start, len, env)
	}

	pub fn set(
		&self,
		start: usize,
		len: usize,
		replacement: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		self.0.set(self, start, len, replacement, env)
	}
}
