use crate::containers::{MaybeSendSync, RefCount};
use crate::value::integer::IntType;
use crate::value::text::Encoding;
use crate::value::{
	Boolean, Integer, List, NamedType, Runnable, Text, ToBoolean, ToInteger, ToList, ToText,
};
use crate::{Environment, Error, Result, Value};
use std::cmp::Ordering;
use std::fmt::Display;

/// A Custom type
pub type Custom<'a, I, E> = RefCount<dyn CustomType<'a, I, E>>;

pub fn new<'e, I, E, T: CustomType<'e, I, E> + 'static>(data: T) -> Custom<'e, I, E> {
	RefCount::new(data)
}

#[allow(unused_variables)]
pub trait CustomType<'e, I, E>: std::fmt::Debug + MaybeSendSync {
	fn to_custom(self: RefCount<Self>) -> Custom<'e, I, E>;

	fn typename(&self) -> &'static str {
		std::any::type_name::<Self>()
	}

	fn eql(&self, rhs: &Value<'e, I, E>) -> bool {
		match rhs {
			Value::Custom(cust) => (self as *const Self).cast::<u8>() == RefCount::as_ptr(cust).cast(),
			_ => false,
		}
	}

	fn run(self: RefCount<Self>, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>> {
		Ok(self.to_custom().into())
	}

	fn to_text(self: RefCount<Self>, env: &mut Environment<'e, I, E>) -> Result<Text<E>>
	where
		E: Encoding,
		I: Display,
	{
		Err(Error::NoConversion { to: Text::<E>::TYPENAME, from: self.typename() })
	}

	fn to_integer(self: RefCount<Self>, env: &mut Environment<'e, I, E>) -> Result<Integer<I>>
	where
		I: IntType,
	{
		Err(Error::NoConversion { to: Integer::<I>::TYPENAME, from: self.typename() })
	}

	fn to_boolean(self: RefCount<Self>, env: &mut Environment<'e, I, E>) -> Result<Boolean>
	where
		I: IntType,
	{
		Err(Error::NoConversion { to: Boolean::TYPENAME, from: self.typename() })
	}

	fn to_list(self: RefCount<Self>, env: &mut Environment<'e, I, E>) -> Result<List<'e, I, E>>
	where
		I: IntType,
	{
		Err(Error::NoConversion { to: List::<I, E>::TYPENAME, from: self.typename() })
	}

	fn head(self: RefCount<Self>, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "["))
	}

	fn tail(self: RefCount<Self>, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "]"))
	}

	fn length(self: RefCount<Self>, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Integer::<I>::try_from(self.to_list(env)?.len()).map(Value::from)
	}

	fn ascii(self: RefCount<Self>, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "ASCII"))
	}

	fn add(
		self: RefCount<Self>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "+"))
	}

	fn subtract(
		self: RefCount<Self>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "-"))
	}

	fn multiply(
		self: RefCount<Self>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "*"))
	}

	fn divide(
		self: RefCount<Self>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "/"))
	}

	fn remainder(
		self: RefCount<Self>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "%"))
	}

	fn power(
		self: RefCount<Self>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "^"))
	}

	fn compare(
		self: RefCount<Self>,
		rhs: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Ordering>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "<cmp>"))
	}

	fn assign(
		self: RefCount<Self>,
		rhs: Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<()>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "="))
	}

	fn get(
		self: RefCount<Self>,
		start: usize,
		len: usize,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "GET"))
	}

	fn set(
		self: RefCount<Self>,
		start: usize,
		len: usize,
		replacement: &Value<'e, I, E>,
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>>
	where
		I: IntType,
		E: Encoding,
	{
		Err(Error::TypeError(self.typename(), "GET"))
	}
}

impl<'e, I: Display, E: Encoding> ToText<'e, I, E> for Custom<'e, I, E> {
	fn to_text(&self, env: &mut Environment<'e, I, E>) -> Result<Text<E>> {
		CustomType::to_text(self.clone(), env)
	}
}

impl<'e, I: IntType, E> ToInteger<'e, I, E> for Custom<'e, I, E> {
	fn to_integer(&self, env: &mut Environment<'e, I, E>) -> Result<Integer<I>> {
		CustomType::to_integer(self.clone(), env)
	}
}

impl<'e, I: IntType, E> ToBoolean<'e, I, E> for Custom<'e, I, E> {
	fn to_boolean(&self, env: &mut Environment<'e, I, E>) -> Result<Boolean> {
		CustomType::to_boolean(self.clone(), env)
	}
}

impl<'e, I: IntType, E> ToList<'e, I, E> for Custom<'e, I, E> {
	fn to_list(&self, env: &mut Environment<'e, I, E>) -> Result<List<'e, I, E>> {
		CustomType::to_list(self.clone(), env)
	}
}

impl<'e, I, E> Runnable<'e, I, E> for Custom<'e, I, E> {
	fn run(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>> {
		CustomType::run(self.clone(), env)
	}
}
