use crate::value::{
	Boolean, Integer, List, NamedType, Runnable, Text, ToBoolean, ToInteger, ToList, ToText,
};
use crate::{Environment, Error, Result, Value};

use std::cmp::Ordering;

#[cfg(feature = "multithreaded")]
type Inner<T> = std::sync::Arc<T>;

#[cfg(not(feature = "multithreaded"))]
type Inner<T> = std::rc::Rc<T>;

#[derive(Debug, Clone)]
pub struct Custom<'a>(pub Inner<dyn CustomType<'a>>);

impl PartialEq for Custom<'_> {
	fn eq(&self, rhs: &Self) -> bool {
		self.0.eq(&*rhs.0)
	}
}

pub trait CustomType<'e>: std::fmt::Debug {
	fn typename(&self) -> &'static str;

	fn eq(&self, rhs: &dyn CustomType<'e>) -> bool {
		// std::ptr::eq(self , rhs)
		let _ = rhs;
		todo!();
	}

	fn run(&self, this: &Custom<'e>, _env: &mut Environment<'e>) -> Result<Value<'e>> {
		Ok(this.clone().into())
	}

	fn to_text(&self, _this: &Custom<'e>) -> Result<Text> {
		Err(Error::NoConversion { to: Text::TYPENAME, from: self.typename() })
	}

	fn to_integer(&self, _this: &Custom<'e>) -> Result<Integer> {
		Err(Error::NoConversion { to: Integer::TYPENAME, from: self.typename() })
	}

	fn to_boolean(&self, _this: &Custom<'e>) -> Result<Boolean> {
		Err(Error::NoConversion { to: Boolean::TYPENAME, from: self.typename() })
	}

	fn to_list(&self, _this: &Custom<'e>) -> Result<List<'e>> {
		Err(Error::NoConversion { to: List::TYPENAME, from: self.typename() })
	}

	fn head(&self, _this: &Custom<'e>, env: &mut Environment<'e>) -> Result<Value<'e>> {
		let _ = env;
		Err(Error::TypeError(self.typename(), "["))
	}

	fn tail(&self, _this: &Custom<'e>, env: &mut Environment<'e>) -> Result<Value<'e>> {
		let _ = env;
		Err(Error::TypeError(self.typename(), "]"))
	}

	fn length(&self, _this: &Custom<'e>, env: &mut Environment<'e>) -> Result<Value<'e>> {
		let _ = env;
		Integer::try_from(self.to_list(_this)?.len()).map(Value::from)
	}

	fn ascii(&self, _this: &Custom<'e>, env: &mut Environment<'e>) -> Result<Value<'e>> {
		let _ = env;
		Err(Error::TypeError(self.typename(), "ASCII"))
	}

	fn add(
		&self,
		_this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e>,
	) -> Result<Value<'e>> {
		let _ = (rhs, env);
		Err(Error::TypeError(self.typename(), "+"))
	}

	fn subtract(
		&self,
		_this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e>,
	) -> Result<Value<'e>> {
		let _ = (rhs, env);
		Err(Error::TypeError(self.typename(), "-"))
	}

	fn multiply(
		&self,
		_this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e>,
	) -> Result<Value<'e>> {
		let _ = (rhs, env);
		Err(Error::TypeError(self.typename(), "*"))
	}

	fn divide(
		&self,
		_this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e>,
	) -> Result<Value<'e>> {
		let _ = (rhs, env);
		Err(Error::TypeError(self.typename(), "/"))
	}

	fn remainder(
		&self,
		_this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e>,
	) -> Result<Value<'e>> {
		let _ = (rhs, env);
		Err(Error::TypeError(self.typename(), "%"))
	}

	fn power(
		&self,
		_this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e>,
	) -> Result<Value<'e>> {
		let _ = (rhs, env);
		Err(Error::TypeError(self.typename(), "^"))
	}

	fn compare(
		&self,
		_this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e>,
	) -> Result<Ordering> {
		let _ = (rhs, env);
		Err(Error::TypeError(self.typename(), "<cmp>"))
	}

	fn equals(
		&self,
		_this: &Custom<'e>,
		rhs: &Value<'e>,
		env: &mut Environment<'e>,
	) -> Result<bool> {
		let _ = env;

		if let Value::Custom(rhs) = rhs {
			Ok(self.eq(&*rhs.0))
		} else {
			Ok(false)
		}
	}

	fn assign(&self, _this: &Custom<'e>, rhs: Value<'e>, env: &mut Environment<'e>) -> Result<()> {
		let _ = (rhs, env);
		Err(Error::TypeError(self.typename(), "ASSIGN"))
	}

	fn get(
		&self,
		_this: &Custom<'e>,
		start: usize,
		len: usize,
		env: &mut Environment<'e>,
	) -> Result<Value<'e>> {
		let _ = (start, len, env);
		Err(Error::TypeError(self.typename(), "GET"))
	}

	fn set(
		&self,
		_this: &Custom<'e>,
		start: usize,
		len: usize,
		replacement: &Value<'e>,
		env: &mut Environment<'e>,
	) -> Result<Value<'e>> {
		let _ = (start, len, replacement, env);
		Err(Error::TypeError(self.typename(), "GET"))
	}
}

impl ToText for Custom<'_> {
	fn to_text(&self) -> Result<Text> {
		self.0.to_text(self)
	}
}

impl ToInteger for Custom<'_> {
	fn to_integer(&self) -> Result<Integer> {
		self.0.to_integer(self)
	}
}

impl ToBoolean for Custom<'_> {
	fn to_boolean(&self) -> Result<Boolean> {
		self.0.to_boolean(self)
	}
}

impl<'e> ToList<'e> for Custom<'e> {
	fn to_list(&self) -> Result<List<'e>> {
		self.0.to_list(self)
	}
}

impl<'e> Runnable<'e> for Custom<'e> {
	fn run(&self, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.run(self, env)
	}
}

impl<'e> Custom<'e> {
	pub fn typename(&self) -> &'static str {
		self.0.typename()
	}

	pub fn eq(&self, rhs: &dyn CustomType<'e>) -> bool {
		self.0.eq(rhs)
	}

	pub fn run(&self, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.run(self, env)
	}

	pub fn head(&self, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.head(self, env)
	}

	pub fn tail(&self, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.tail(self, env)
	}

	pub fn length(&self, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.length(self, env)
	}

	pub fn ascii(&self, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.ascii(self, env)
	}

	pub fn add(&self, rhs: &Value<'e>, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.add(self, rhs, env)
	}

	pub fn subtract(&self, rhs: &Value<'e>, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.subtract(self, rhs, env)
	}

	pub fn multiply(&self, rhs: &Value<'e>, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.multiply(self, rhs, env)
	}

	pub fn divide(&self, rhs: &Value<'e>, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.divide(self, rhs, env)
	}

	pub fn remainder(&self, rhs: &Value<'e>, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.remainder(self, rhs, env)
	}

	pub fn power(&self, rhs: &Value<'e>, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.power(self, rhs, env)
	}

	pub fn compare(&self, rhs: &Value<'e>, env: &mut Environment<'e>) -> Result<Ordering> {
		self.0.compare(self, rhs, env)
	}

	pub fn equals(&self, rhs: &Value<'e>, env: &mut Environment<'e>) -> Result<bool> {
		self.0.equals(self, rhs, env)
	}

	pub fn assign(&self, rhs: Value<'e>, env: &mut Environment<'e>) -> Result<()> {
		self.0.assign(self, rhs, env)
	}

	pub fn get(&self, start: usize, len: usize, env: &mut Environment<'e>) -> Result<Value<'e>> {
		self.0.get(self, start, len, env)
	}

	pub fn set(
		&self,
		start: usize,
		len: usize,
		replacement: &Value<'e>,
		env: &mut Environment<'e>,
	) -> Result<Value<'e>> {
		self.0.set(self, start, len, replacement, env)
	}
}
