#![allow(non_snake_case)]
use crate::env::Flags;
use crate::parse::{self, Parsable, Parser};
use crate::value::integer::IntType;
use crate::value::text::{Character, Encoding, TextSlice};
#[cfg(feature = "extensions")]
use crate::value::Text;
use crate::value::{List, Runnable, ToBoolean, ToInteger, ToText};
use crate::{Environment, Error, RefCount, Result, Value};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::Write;

/// A runnable function in Knight, e.g. `+`.
pub struct Function<'a, I, E>(RefCount<Inner<'a, I, E>>);
impl<I, E> Clone for Function<'_, I, E> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

struct Inner<'a, I, E> {
	func: FnType<I, E>,
	full_name: &'a TextSlice<E>,
	short_name: Option<Character<E>>,
	arity: usize,
}

type AllocFn<I, E> = dyn for<'e> Fn(&[Value<'e, I, E>], &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>>
	+ Send
	+ Sync
	+ 'static;

pub enum FnType<I, E> {
	FnPtr(for<'e> fn(&[Value<'e, I, E>], &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>>),
	Alloc(Box<AllocFn<I, E>>),
}

impl<I, E> Eq for Function<'_, I, E> {}
impl<I, E> PartialEq for Function<'_, I, E> {
	/// Functions are only equal if they're identical.
	fn eq(&self, rhs: &Self) -> bool {
		RefCount::ptr_eq(&self.0, &rhs.0)
	}
}

impl<I, E> Hash for Function<'_, I, E> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.short_name().expect("<invalid function>").hash(state)
	}
}

impl<I, E> Borrow<Character<E>> for Function<'_, I, E> {
	fn borrow(&self) -> &Character<E> {
		self.0.short_name.as_ref().expect("<invalid function>")
	}
}

impl<I, E> Debug for Function<'_, I, E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("Function")
				.field("name", &self.full_name())
				.field("arity", &self.arity())
				// .field("fnptr", &(self.func.0 as usize as *const ()))
				.finish()
		} else {
			f.debug_tuple("Function").field(&self.full_name()).finish()
		}
	}
}

pub struct ExtensionFunction<'a, I, E>(pub Function<'a, I, E>);
impl<I, E> Clone for ExtensionFunction<'_, I, E> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<I, E> Eq for ExtensionFunction<'_, I, E> {}
impl<I, E> PartialEq for ExtensionFunction<'_, I, E> {
	fn eq(&self, rhs: &Self) -> bool {
		self.0 == rhs.0
	}
}
impl<I, E> Debug for ExtensionFunction<'_, I, E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("ExtensionFunction")
				.field("name", &self.0.full_name())
				.field("arity", &self.0.arity())
				// .field("fnptr", &(self.func.0 as usize as *const ()))
				.finish()
		} else {
			f.debug_tuple("ExtensionFunction").field(&self.0.full_name()).finish()
		}
	}
}

impl<I, E> Hash for ExtensionFunction<'_, I, E> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0 .0.full_name.hash(state)
	}
}

impl<I, E> Borrow<TextSlice<E>> for ExtensionFunction<'_, I, E> {
	fn borrow(&self) -> &TextSlice<E> {
		self.0 .0.full_name
	}
}

impl<'e, I: IntType, E: Encoding> Parsable<'e, I, E> for Function<'e, I, E> {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, 'e, I, E>) -> parse::Result<Option<Self>> {
		// FIXME: make this parsing part of the extension function itself
		#[cfg(feature = "extensions")]
		if parser.peek().map_or(false, |chr| chr == 'X') {
			let name = parser.take_while(crate::value::text::Character::is_upper).unwrap();

			return parser.env().extensions().get(name).cloned().map(|e| Some(e.0)).ok_or_else(|| {
				parser.error(parse::ErrorKind::UnknownExtensionFunction(name.to_string()))
			});
		}

		let Some(head) = parser.peek() else {
			return Ok(None);
		};

		let Some(function) = parser.env().functions().get(&head).cloned() else {
			return Ok(None);
		};

		parser.strip_function();

		Ok(Some(function))
	}
}

impl<'a, I, E> Function<'a, I, E> {
	// #[must_use]
	// pub const fn new_const(full_name: &'a TextSlice, arity: usize, func: FnType) -> Self {
	// 	Self {
	// 		full_name,
	// 		arity,
	// 		func,
	// 		short_name: Some(unsafe {
	// 			Character::new_unchecked(full_name.as_str().as_bytes()[0] as char)
	// 		}),
	// 	}
	// }

	#[must_use]
	pub fn new<F>(full_name: &'a TextSlice<E>, arity: usize, func: F) -> Self
	where
		F: for<'e> Fn(&[Value<'e, I, E>], &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>>
			+ Send
			+ Sync
			+ 'static,
	{
		Self(RefCount::from(Inner {
			full_name,
			arity,
			func: FnType::Alloc(Box::new(func) as _),
			short_name: Some(unsafe {
				Character::new_unchecked(full_name.as_str().as_bytes()[0] as char)
			}),
		}))
	}

	/// The long-hand name of this function.
	///
	/// For extension functions that start with `X`, this should also start with it.
	#[must_use]
	pub fn full_name(&self) -> &'a TextSlice<E> {
		&self.0.full_name
	}

	/// Gets the shorthand name for `self`. Returns `None` if it's an `X` function.
	#[must_use]
	pub fn short_name(&self) -> Option<Character<E>> {
		self.0.short_name
	}

	/// The arity of the function, i.e. how many arguments it takes.
	#[must_use]
	pub fn arity(&self) -> usize {
		self.0.arity
	}
}

impl<'a, I: IntType, E: Encoding> Function<'a, I, E> {
	/// Executes this function
	pub fn run<'e>(
		&self,
		args: &[Value<'e, I, E>],
		env: &mut Environment<'e, I, E>,
	) -> Result<Value<'e, I, E>> {
		debug_assert_eq!(args.len(), self.arity());

		match self.0.func {
			FnType::FnPtr(fnptr) => fnptr(args, env),
			FnType::Alloc(ref alloc) => alloc(args, env),
		}
	}

	pub(crate) fn default_set(flags: &Flags) -> HashSet<Self> {
		let mut map = HashSet::new();

		macro_rules! insert {
			($($(#[$meta:meta] $feature:ident)? $name:ident)*) => {$(
				$(#[$meta])?
				if true $(&& flags.exts.fns.$feature)? {
					map.insert($name());
				}
			)*}
		}

		insert! {
			PROMPT RANDOM
			BLOCK CALL QUIT NOT NEG LENGTH DUMP OUTPUT ASCII BOX HEAD TAIL
			ADD SUBTRACT MULTIPLY DIVIDE REMAINDER POWER EQUALS LESS_THAN GREATER_THAN AND OR
				THEN ASSIGN WHILE
			IF GET SET

			#[cfg(feature = "extensions")] value VALUE
			#[cfg(feature = "extensions")] eval EVAL
			#[cfg(feature = "extensions")] handle HANDLE
			#[cfg(feature = "extensions")] yeet YEET
			#[cfg(feature = "extensions")] r#use USE
			#[cfg(feature = "extensions")] system SYSTEM
		}

		let _ = flags;
		map
	}
}

#[cfg(feature = "extensions")]
impl<'e, I: IntType, E: Encoding> ExtensionFunction<'e, I, E> {
	pub(crate) fn default_set(flags: &Flags) -> HashSet<Self> {
		let mut map = HashSet::new();

		macro_rules! insert {
			($($feature:ident $name:ident)*) => {
				$(
					if flags.exts.fns.$feature {
						map.insert($name());
					}
				)*
			}
		}

		insert! {
			xsrand XSRAND
			xreverse XREVERSE
			xrange XRANGE
		}

		map
	}
}

macro_rules! arity {
	() => (0);
	($_pat:ident $($rest:ident)*) => (1+arity!($($rest)*))
}
macro_rules! function {
	($name:literal, $env:pat, |$($args:ident),*| $body:block) => {
		Function(RefCount::from(Inner{
			full_name: unsafe { TextSlice::new_unchecked($name) },
			arity: arity!($($args)*),
			short_name:
			Some(unsafe { Character::new_unchecked(TextSlice::<E>::new_unchecked($name).as_str().as_bytes()[0] as char) }),
			func: FnType::FnPtr(|args, $env| {
				let [$($args,)*]: &[Value::<'_, I, E>; arity!($($args)*)] = args.try_into().unwrap();
				Ok($body)
			})
		}))
	};
}

#[cfg_attr(not(feature = "extensions"), allow(unused_macros))]
macro_rules! xfunction {
	($($tt:tt)*) => {
		ExtensionFunction(function!($($tt)*))
	}
}

/// **4.1.4**: `PROMPT`
pub fn PROMPT<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("PROMPT", env, |/* comment for rustfmt */| {
	env.read_line()?.map(Value::from).unwrap_or_default()
})
}

/// **4.1.5**: `RANDOM`
pub fn RANDOM<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("RANDOM", env, |/* comment for rustfmt */| {
	// note that `env.random()` is seedable with `XSRAND`
	env.random().into()
})
}

/// **4.2.2** `BOX`
pub fn BOX<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!(",", env, |val| {
		// `boxed` is optimized over `vec![val.run(env)]`
		List::boxed(val.run(env)?).into()
	})
}

pub fn HEAD<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("[", env, |val| {
		// <comment for a single line>
		val.run(env)?.head(env)?
	})
}

pub fn TAIL<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("]", env, |val| {
		// <comment for a single line>
		val.run(env)?.tail(env)?
	})
}

/// **4.2.3** `BLOCK`  
pub fn BLOCK<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("BLOCK", env, |arg| {
		// Technically, according to the spec, only the return value from `BLOCK` can be used in `CALL`.
		// Since this function normally just returns whatever it's argument is, it's impossible to
		// distinguish an `Integer` returned from `BLOCK` and one simply given to `CALL`. As such, when
		// the `strict-call-argument` feature is enabled. We ensure that we _only_ return `Ast`s
		// from `BLOCK`, so `CALL` can verify them.
		#[cfg(feature = "compliance")]
		if env.flags().compliance.check_call_arg && !matches!(arg, Value::Ast(_)) {
			// The NOOP function literally just runs its argument.
			fn NOOP<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
				function!(":", env, |arg| {
					debug_assert!(!matches!(arg, Value::Ast(_)));

					arg.run(env)? // We can't simply `.clone()` the arg in case we're given a variable name.
				})
			}

			return Ok(crate::Ast::new(NOOP(), vec![arg.clone()].into()).into());
		}

		let _ = env;
		arg.clone()
	})
}

/// **4.2.4** `CALL`  
pub fn CALL<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("CALL", env, |arg| {
		//
		arg.run(env)?.call(env)?
	})
}

/// **4.2.6** `QUIT`  
pub fn QUIT<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("QUIT", env, |arg| {
		let status = arg.run(env)?.to_integer(env)?;

		match i32::try_from(status) {
			Ok(status)
				if {
					#[cfg(feature = "compliance")]
					{
						!env.flags().compliance.check_quit_bounds
					}
					#[cfg(not(feature = "compliance"))]
					{
						true
					}
				} || (0..=127).contains(&status) =>
			{
				return Err(Error::Quit(status))
			}

			_ => return Err(Error::DomainError("exit code out of bounds")),
		}

		// The `function!` macro calls `Ok(...)` on the return value of this block,
		// so we need _something_ here so it can typecheck correctly.
		#[allow(unreachable_code)]
		Value::Null
	})
}

/// **4.2.7** `!`  
pub fn NOT<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("!", env, |arg| {
		// <blank line so rustfmt doesnt wrap onto the prev line>
		(!arg.run(env)?.to_boolean(env)?).into()
	})
}

/// **4.2.8** `LENGTH`  
pub fn LENGTH<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("LENGTH", env, |arg| {
		//
		arg.run(env)?.length(env)?
	})
}

/// **4.2.9** `DUMP`  
pub fn DUMP<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("DUMP", env, |arg| {
		let value = arg.run(env)?;
		write!(env.output(), "{value:?}")?;
		value
	})
}

/// **4.2.10** `OUTPUT`  
pub fn OUTPUT<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("OUTPUT", env, |arg| {
		let text = arg.run(env)?.to_text(env)?;
		let output = env.output();

		if let Some(stripped) = text.strip_suffix('\\') {
			write!(output, "{stripped}")?
		} else {
			writeln!(output, "{text}")?;
		}

		output.flush()?;

		Value::Null
	})
}

/// **4.2.11** `ASCII`  
pub fn ASCII<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("ASCII", env, |arg| {
		//
		arg.run(env)?.ascii(env)?
	})
}

/// **4.2.12** `~`  
pub fn NEG<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("~", env, |arg| {
		// comment so it wont make it one line
		arg.run(env)?.to_integer(env)?.negate()?.into()
	})
}

/// **4.3.1** `+`  
pub fn ADD<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("+", env, |lhs, rhs| {
		//
		lhs.run(env)?.add(&rhs.run(env)?, env)?
	})
}

/// **4.3.2** `-`  
pub fn SUBTRACT<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("-", env, |lhs, rhs| {
		//
		lhs.run(env)?.subtract(&rhs.run(env)?, env)?
	})
}

/// **4.3.3** `*`  
pub fn MULTIPLY<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("*", env, |lhs, rhs| {
		//
		lhs.run(env)?.multiply(&rhs.run(env)?, env)?
	})
}

/// **4.3.4** `/`  
pub fn DIVIDE<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("/", env, |lhs, rhs| {
		//
		lhs.run(env)?.divide(&rhs.run(env)?, env)?
	})
}

/// **4.3.5** `%`  
pub fn REMAINDER<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("%", env, |lhs, rhs| {
		//
		lhs.run(env)?.remainder(&rhs.run(env)?, env)?
	})
}

/// **4.3.6** `^`  
pub fn POWER<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("^", env, |lhs, rhs| {
		//
		lhs.run(env)?.power(&rhs.run(env)?, env)?
	})
}

/// **4.3.7** `<`  
pub fn LESS_THAN<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("<", env, |lhs, rhs| {
		(lhs.run(env)?.compare(&rhs.run(env)?, env)? == Ordering::Less).into()
	})
}

/// **4.3.8** `>`  
pub fn GREATER_THAN<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!(">", env, |lhs, rhs| {
		(lhs.run(env)?.compare(&rhs.run(env)?, env)? == Ordering::Greater).into()
	})
}

/// **4.3.9** `?`  
pub fn EQUALS<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("?", env, |lhs, rhs| {
		//
		lhs.run(env)?.equals(&rhs.run(env)?, env)?.into()
	})
}

/// **4.3.10** `&`  
pub fn AND<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("&", env, |lhs, rhs| {
		let condition = lhs.run(env)?;

		if condition.to_boolean(env)? {
			rhs.run(env)?
		} else {
			condition
		}
	})
}

/// **4.3.11** `|`  
pub fn OR<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("|", env, |lhs, rhs| {
		let condition = lhs.run(env)?;

		if condition.to_boolean(env)? {
			condition
		} else {
			rhs.run(env)?
		}
	})
}

/// **4.3.12** `;`  
pub fn THEN<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!(";", env, |lhs, rhs| {
		lhs.run(env)?;
		rhs.run(env)?
	})
}

/// **4.3.13** `=`  
pub fn ASSIGN<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("=", env, |variable, value| {
		let ran = value.run(env)?;
		variable.assign(ran.clone(), env)?;
		ran
	})
}

/// **4.3.14** `WHILE`  
pub fn WHILE<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("WHILE", env, |condition, body| {
		while condition.run(env)?.to_boolean(env)? {
			body.run(env)?;
		}

		Value::Null
	})
}

/// **4.4.1** `IF`  
pub fn IF<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("IF", env, |condition, iftrue, iffalse| {
		if condition.run(env)?.to_boolean(env)? {
			iftrue.run(env)?
		} else {
			iffalse.run(env)?
		}
	})
}

/// **4.4.2** `GET`  
pub fn GET<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("GET", env, |source, start, length| {
		//
		source.run(env)?.get(&start.run(env)?, &length.run(env)?, env)?
	})
}

/// **4.5.1** `SET`  
pub fn SET<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("SET", env, |source, start, length, replacement| {
		//
		source.run(env)?.set(&start.run(env)?, &length.run(env)?, &replacement.run(env)?, env)?
	})
}

/// The `VALUE` extension function.
///
/// This takes a single argument, converts it to a [`Text`](crate::value::Text) and interprets it
/// as a variable name. Then, it looks up the last assigned value to that variable.
#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn VALUE<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("VALUE", env, |arg| {
		let name = arg.run(env)?.to_text(env)?;
		env.lookup(&name)?.into()
	})
}

#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn HANDLE<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("HANDLE", env, |block, iferr| {
		let err_var_name = unsafe { crate::TextSlice::new_unchecked("_") };

		match block.run(env) {
			Ok(value) => value,
			Err(err) => {
				// This is fallible, as the error string might have had something bad.
				let errmsg = Text::new(err.to_string(), env.flags())?;

				// Assign it to the error variable
				env.lookup(err_var_name).unwrap().assign(errmsg.into());

				// Finally, execute the RHS.
				iferr.run(env)?
			}
		}
	})
}

#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn YEET<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("YEET", env, |errmsg| {
		return Err(Error::Custom(errmsg.run(env)?.to_text(env)?.to_string().into()));

		#[allow(unreachable_code)]
		Value::Null
	})
}

/// **6.3** `USE`
#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn USE<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("USE", env, |arg| {
		let filename = arg.run(env)?.to_text(env)?;
		let contents = env.read_file(&filename)?;

		env.play(&contents)?
	})
}

/// **4.2.2** `EVAL`
#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn EVAL<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("EVAL", env, |val| {
		let code = val.run(env)?.to_text(env)?;
		env.play(&code)?
	})
}

/// **4.2.5** `` ` ``
#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn SYSTEM<'e, I: IntType, E: Encoding>() -> Function<'e, I, E> {
	function!("$", env, |cmd, stdin| {
		let command = cmd.run(env)?.to_text(env)?;
		let stdin = match stdin.run(env)? {
			Value::Text(text) => Some(text),
			Value::Null => None,
			other => return Err(Error::TypeError(other.typename(), "$")),
		};

		env.run_command(&command, stdin.as_deref())?.into()
	})
}

/// **Compiler extension**: SRAND
#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn XSRAND<'e, I: IntType, E: Encoding>() -> ExtensionFunction<'e, I, E> {
	xfunction!("XSRAND", env, |arg| {
		let seed = arg.run(env)?.to_integer(env)?;
		env.srand(seed);
		Value::Null
	})
}

/// **Compiler extension**: REV
#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn XREVERSE<'e, I: IntType, E: Encoding>() -> ExtensionFunction<'e, I, E> {
	xfunction!("XREVERSE", env, |arg| {
		match arg.run(env)? {
			Value::Text(_text) => {
				// text.chars().collect::<Vec<Character>>().into_iter().rev().collect::<Text>().into()
				todo!()
			}
			Value::List(list) => {
				let mut eles = list.iter().cloned().collect::<Vec<Value<'_, I, E>>>();
				eles.reverse();
				List::new(eles, env.flags()).unwrap().into()
			}
			other => return Err(Error::TypeError(other.typename(), "XRANGE")),
		}
	})
}

#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn XRANGE<'e, I: IntType, E: Encoding>() -> ExtensionFunction<'e, I, E> {
	xfunction!("XRANGE", env, |start, stop| {
		match start.run(env)? {
			Value::Integer(start) => {
				let stop = stop.run(env)?.to_integer(env)?;

				match start <= stop {
					true => List::new(
						(i64::from(start)..i64::from(stop))
							.map(|x| Value::from(crate::value::Integer::try_from(x).unwrap()))
							.collect::<Vec<Value<'_, I, E>>>(),
						env.flags(),
					)
					.expect("todo: out of bounds error")
					.into(),

					false => {
						// (stop..start).map(Value::from).rev().collect::<List>().into()
						todo!()
					}
				}
			}

			Value::Text(_text) => {
				// let start = text.get(0).a;
				todo!()
			}

			other => return Err(Error::TypeError(other.typename(), "XRANGE")),
		}
	})
}
