// #![warn(missing_docs, missing_doc_code_examples)]
#![allow(clippy::tabs_in_doc_comments, unused)]
#![warn(/*, missing_doc_code_examples, missing_docs*/)]

pub mod function;
pub mod rcstring;
mod value;
mod error;
mod stream;
pub mod environment;

/// The number type within Knight.
pub type Number = i64;

#[doc(inline)]
pub use rcstring::RcString;

#[doc(inline)]
pub use function::Function;

pub use stream::Stream;
pub use environment::{Environment, Variable};
pub use value::Value;
pub use error::{ParseError, RuntimeError};

/// Runs the given string as Knight code, returning the result of its execution.
pub fn run_str<S, I, O>(input: S, env: &mut Environment<I, O>) -> Result<Value<I, O>, RuntimeError>
where
	S: AsRef<str>,
	I: std::io::Read,
	O: std::io::Write
{
	run(input.as_ref().chars(), env)
}

/// Parses a [`Value`] from the given iterator and then runs the value.
pub fn run<S, I, O>(input: S, env: &mut Environment<I, O>) -> Result<Value<I, O>, RuntimeError>
where
	S: IntoIterator<Item=char>,
	I: std::io::Read,
	O: std::io::Write
{
	Value::parse(input, env)?.run(env)
}
