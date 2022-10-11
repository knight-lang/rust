use crate::value::{
	Boolean, Integer, List, NamedType, Runnable, Text, ToBoolean, ToInteger, ToList, ToText,
};
use crate::{containers::MaybeSendSync, Environment, Error, RefCount, Result, Value};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Custom<'a>(RefCount<dyn CustomType<'a>>);

impl PartialEq for Custom<'_> {
	fn eq(&self, rhs: &Self) -> bool {
		self.0.eq(&*rhs.0)
	}
}

impl<'a> Custom<'a> {
	pub fn new<T: CustomType<'a> + 'static>(data: T) -> Self {
		Self((Box::new(data) as Box<dyn CustomType<'a>>).into())
	}
}

pub trait CustomType<'e>: std::fmt::Debug + MaybeSendSync {
	fn typename(&self) -> &'static str {
		std::any::type_name::<Self>()
	}

	#[allow(clippy::should_implement_trait)]
	fn eq(&self, rhs: &dyn CustomType<'e>) -> bool {
		// std::ptr::eq(self as *const u8, rhs as *const u8)
		let _ = rhs;
		todo!();
	}

	fn run(&self, this: &Custom<'e>, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		let _ = env;
		Ok(this.clone().into())
	}

	fn to_text(&self, this: &Custom<'e>, env: &mut Environment<'e, I, E>) -> Result<Text> {
		let _ = (this, env);
		Err(Error::NoConversion { to: Text::TYPENAME, from: self.typename() })
	}

	fn to_integer(&self, this: &Custom<'e>, env: &mut Environment<'e, I, E>) -> Result<Integer> {
		let _ = (this, env);
		Err(Error::NoConversion { to: Integer::TYPENAME, from: self.typename() })
	}

	fn to_boolean(&self, this: &Custom<'e>, env: &mut Environment<'e, I, E>) -> Result<Boolean> {
		let _ = (this, env);
		Err(Error::NoConversion { to: Boolean::TYPENAME, from: self.typename() })
	}

	fn to_list(&self, this: &Custom<'e>, env: &mut Environment<'e, I, E>) -> Result<List<'e>> {
		let _ = (this, env);
		Err(Error::NoConversion { to: List::TYPENAME, from: self.typename() })
	}

	fn head(&self, this: &Custom<'e>, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		let _ = (this, env);
		Err(Error::TypeError(self.typename(), "["))
	}

	fn tail(&self, this: &Custom<'e>, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		let _ = (this, env);
		Err(Error::TypeError(self.typename(), "]"))
	}

	fn length(&self, this: &Custom<'e>, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		Integer::try_from(self.to_list(this, env)?.len()).map(Value::from)
	}

	fn ascii(&self, this: &Custom<'e>, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		let _ = (this, env);
		Err(Error::TypeError(self.typename(), "ASCII"))
	}

	fn add(
		&self,
		this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "+"))
	}

	fn subtract(
		&self,
		this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "-"))
	}

	fn multiply(
		&self,
		this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "*"))
	}

	fn divide(
		&self,
		this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "/"))
	}

	fn remainder(
		&self,
		this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "%"))
	}

	fn power(
		&self,
		this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e>> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "^"))
	}

	fn compare(
		&self,
		this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Ordering> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "<cmp>"))
	}

	fn assign(
		&self,
		this: &Custom<'e>,
		rhs: Value<'e>,
		env: &mut Environment<'e, I, E>,
	) -> Result<()> {
		let _ = (this, rhs, env);
		Err(Error::TypeError(self.typename(), "ASSIGN"))
	}

	fn get(
		&self,
		this: &Custom<'e>,
		start: usize,
		len: usize,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e>> {
		let _ = (this, start, len, env);
		Err(Error::TypeError(self.typename(), "GET"))
	}

	fn set(
		&self,
		this: &Custom<'e>,
		start: usize,
		len: usize,
		replacement: &Value<'e>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e>> {
		let _ = (this, start, len, replacement, env);
		Err(Error::TypeError(self.typename(), "GET"))
	}
}

impl<'e> ToText<'e> for Custom<'e> {
	fn to_text(&self, env: &mut Environment<'e, I, E>) -> Result<Text> {
		self.0.to_text(self, env)
	}
}

impl<'e, I: crate::value::integer::IntType> ToInteger<'e, I, E> for Custom<'e> {
	fn to_integer(&self, env: &mut Environment<'e, I, E>) -> Result<Integer<I>> {
		self.0.to_integer(self, env)
	}
}

impl<'e> ToBoolean<'e> for Custom<'e> {
	fn to_boolean(&self, env: &mut Environment<'e, I, E>) -> Result<Boolean> {
		self.0.to_boolean(self, env)
	}
}

impl<'e> ToList<'e> for Custom<'e> {
	fn to_list(&self, env: &mut Environment<'e, I, E>) -> Result<List<'e>> {
		self.0.to_list(self, env)
	}
}

impl<'e> Runnable<'e> for Custom<'e> {
	fn run(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.run(self, env)
	}
}

impl<'e> Custom<'e> {
	pub fn typename(&self) -> &'static str {
		self.0.typename()
	}

	pub fn run(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.run(self, env)
	}

	pub fn head(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.head(self, env)
	}

	pub fn tail(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.tail(self, env)
	}

	pub fn length(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.length(self, env)
	}

	pub fn ascii(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.ascii(self, env)
	}

	pub fn add(&self, rhs: &Value<'e>, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.add(self, rhs, env)
	}

	pub fn subtract(&self, rhs: &Value<'e>, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.subtract(self, rhs, env)
	}

	pub fn multiply(&self, rhs: &Value<'e>, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.multiply(self, rhs, env)
	}

	pub fn divide(&self, rhs: &Value<'e>, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.divide(self, rhs, env)
	}

	pub fn remainder(&self, rhs: &Value<'e>, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.remainder(self, rhs, env)
	}

	pub fn power(&self, rhs: &Value<'e>, env: &mut Environment<'e, I, E>) -> Result<Value<'e>> {
		self.0.power(self, rhs, env)
	}

	pub fn compare(&self, rhs: &Value<'e>, env: &mut Environment<'e, I, E>) -> Result<Ordering> {
		self.0.compare(self, rhs, env)
	}

	pub fn assign(&self, rhs: Value<'e>, env: &mut Environment<'e, I, E>) -> Result<()> {
		self.0.assign(self, rhs, env)
	}

	pub fn get(
		&self,
		start: usize,
		len: usize,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e>> {
		self.0.get(self, start, len, env)
	}

	pub fn set(
		&self,
		start: usize,
		len: usize,
		replacement: &Value<'e>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e>> {
		self.0.set(self, start, len, replacement, env)
	}
}
