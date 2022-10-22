#![allow(non_snake_case)]
use crate::env::Flags;
use crate::parse::{self, Parsable, Parser};
use crate::value::integer::IntType;
use crate::value::text::TextSlice;
use crate::value::{List, Runnable, Text, ToBoolean, ToInteger, ToText};
use crate::{Environment, Error, RefCount, Result, Value};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::Write;

#[cfg(all(feature = "extensions", feature = "multithreaded"))]
mod fork;

/// A runnable function in Knight, e.g. `+`.
#[derive_where(Clone)]
pub struct Function<I>(RefCount<Inner<I>>);

struct Inner<I> {
	func: FnType<I>,
	full_name: Text,
	short_name: Option<char>,
	arity: usize,
}

type AllocFn<I> =
	dyn Fn(&[Value<I>], &mut Environment<'_, I>) -> Result<Value<I>> + Send + Sync + 'static;

pub enum FnType<I> {
	FnPtr(fn(&[Value<I>], &mut Environment<'_, I>) -> Result<Value<I>>),
	Alloc(Box<AllocFn<I>>),
}

impl<I> Eq for Function<I> {}
impl<I> PartialEq for Function<I> {
	/// Functions are only equal if they're identical.
	fn eq(&self, rhs: &Self) -> bool {
		RefCount::ptr_eq(&self.0, &rhs.0)
	}
}

impl<I> Hash for Function<I> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.short_name().expect("<invalid function>").hash(state)
	}
}

impl<I> Borrow<char> for Function<I> {
	fn borrow(&self) -> &char {
		self.0.short_name.as_ref().expect("<invalid function>")
	}
}

impl<I> Debug for Function<I> {
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

pub struct ExtensionFunction<I>(pub Function<I>);
impl<I> Clone for ExtensionFunction<I> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<I> Eq for ExtensionFunction<I> {}
impl<I> PartialEq for ExtensionFunction<I> {
	fn eq(&self, rhs: &Self) -> bool {
		self.0 == rhs.0
	}
}
impl<I> Debug for ExtensionFunction<I> {
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

impl<I> Hash for ExtensionFunction<I> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0 .0.full_name.hash(state)
	}
}

impl<I> Borrow<TextSlice> for ExtensionFunction<I> {
	fn borrow(&self) -> &TextSlice {
		&self.0 .0.full_name
	}
}

impl<I: IntType> Parsable<I> for Function<I> {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, I>) -> parse::Result<Option<Self>> {
		// FIXME: make this parsing part of the extension function itself
		#[cfg(feature = "extensions")]
		if parser.peek().map_or(false, |chr| chr == 'X') {
			let name = parser.strip_keyword_function().unwrap();

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

		if head.is_uppercase() {
			parser.strip_keyword_function();
		} else {
			parser.advance();
		}

		Ok(Some(function))
	}
}

impl<I> Function<I> {
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
	pub fn new<F>(full_name: Text, arity: usize, func: F) -> Self
	where
		F: Fn(&[Value<I>], &mut Environment<I>) -> Result<Value<I>> + Send + Sync + 'static,
	{
		Self(RefCount::from(Inner {
			arity,
			func: FnType::Alloc(Box::new(func) as _),
			short_name: Some(full_name.head().unwrap()),
			full_name,
		}))
	}

	/// The long-hand name of this function.
	///
	/// For extension functions that start with `X`, this should also start with it.
	#[must_use]
	pub fn full_name(&self) -> &Text {
		&self.0.full_name
	}

	/// Gets the shorthand name for `self`. Returns `None` if it's an `X` function.
	#[must_use]
	pub fn short_name(&self) -> Option<char> {
		self.0.short_name
	}

	/// The arity of the function, i.e. how many arguments it takes.
	#[must_use]
	pub fn arity(&self) -> usize {
		self.0.arity
	}
}

impl<I: IntType> Function<I> {
	/// Executes this function
	pub fn run<'e>(&self, args: &[Value<I>], env: &mut Environment<I>) -> Result<Value<I>> {
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
				if true $(&& flags.extensions.functions.$feature)? {
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
impl<I: IntType> ExtensionFunction<I> {
	pub(crate) fn default_set(flags: &Flags) -> HashSet<Self> {
		let mut map = HashSet::new();

		macro_rules! insert {
			($($feature:ident $name:ident)*) => {
				$(
					if flags.extensions.functions.$feature {
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
			full_name: unsafe { Text::new_unchecked($name) },
			arity: arity!($($args)*),
			short_name: Some(unsafe {TextSlice::new_unchecked($name).as_str().as_bytes()[0] as char }),
			func: FnType::FnPtr(|args, $env| {
				let [$($args,)*]: &[Value::< I>; arity!($($args)*)] = args.try_into().unwrap();
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

/// The `PROMPT` function.
pub fn PROMPT<I: IntType>() -> Function<I> {
	function!("PROMPT", env, |/* comment for rustfmt */| {
		env.prompt().read_line()?.get(env)?.map(Value::from).unwrap_or_default()
	})
}

/// The `RANDOM` function.
pub fn RANDOM<I: IntType>() -> Function<I> {
	function!("RANDOM", env, |/* comment for rustfmt */| {
		// note that `env.random()` is seedable with `XSRAND`
		env.random().into()
	})
}

/// The `BOX` function.
pub fn BOX<I: IntType>() -> Function<I> {
	function!(",", env, |val| {
		// `boxed` is optimized over `vec![val.run(env)]`
		List::boxed(val.run(env)?).into()
	})
}

pub fn HEAD<I: IntType>() -> Function<I> {
	function!("[", env, |val| {
		// <comment for a single line>
		val.run(env)?.head(env)?
	})
}

pub fn TAIL<I: IntType>() -> Function<I> {
	function!("]", env, |val| {
		// <comment for a single line>
		val.run(env)?.tail(env)?
	})
}

/// The `BLOCK` function.
pub fn BLOCK<I: IntType>() -> Function<I> {
	function!("BLOCK", env, |arg| {
		// Technically, according to the spec, only the return value from `BLOCK` can be used in `CALL`.
		// Since this function normally just returns whatever it's argument is, it's impossible to
		// distinguish an `Integer` returned from `BLOCK` and one simply given to `CALL`. As such, when
		// the `strict-call-argument` feature is enabled. We ensure that we _only_ return `Ast`s
		// from `BLOCK`, so `CALL` can verify them.
		#[cfg(feature = "compliance")]
		if env.flags().compliance.check_call_arg && !matches!(arg, Value::Ast(_)) {
			// The NOOP function literally just runs its argument.
			fn NOOP<I: IntType>() -> Function<I> {
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

/// The `CALL` function.
pub fn CALL<I: IntType>() -> Function<I> {
	function!("CALL", env, |arg| {
		//
		arg.run(env)?.call(env)?
	})
}

/// The `QUIT` function.
pub fn QUIT<I: IntType>() -> Function<I> {
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
// /// The `QUIT` function.
// pub fn QUIT<I: IntType>() -> Function<I> {
// 	function!("QUIT", env, |arg| {
// 		let status = arg.run(env)?.to_integer(env)?;

// 		match i32::try_from(status) {
// 			Ok(status)
// 				if {
// 					#[cfg(feature = "compliance")]
// 					{
// 						!env.flags().compliance.check_quit_bounds
// 					}
// 					#[cfg(not(feature = "compliance"))]
// 					{
// 						true
// 					}
// 				} || (0..=127).contains(&status) =>
// 			{
// 				return Err(Error::Quit(status))
// 			}

// 			_ => return Err(Error::DomainError("exit code out of bounds")),
// 		}

// 		// The `function!` macro calls `Ok(...)` on the return value of this block,
// 		// so we need _something_ here so it can typecheck correctly.
// 		#[allow(unreachable_code)]
// 		Value::Null
// 	})
// }

/// The `!` function.
pub fn NOT<I: IntType>() -> Function<I> {
	function!("!", env, |arg| {
		// <blank line so rustfmt doesnt wrap onto the prev line>
		(!arg.run(env)?.to_boolean(env)?).into()
	})
}

/// The `LENGTH` function.
pub fn LENGTH<I: IntType>() -> Function<I> {
	function!("LENGTH", env, |arg| {
		//
		arg.run(env)?.length(env)?
	})
}

/// The `DUMP` function.
pub fn DUMP<I: IntType>() -> Function<I> {
	function!("DUMP", env, |arg| {
		let value = arg.run(env)?;
		write!(env.output(), "{value:?}")?;
		value
	})
}

/// The `OUTPUT` function.
pub fn OUTPUT<I: IntType>() -> Function<I> {
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

/// The `ASCII` function.
pub fn ASCII<I: IntType>() -> Function<I> {
	function!("ASCII", env, |arg| {
		//
		arg.run(env)?.ascii(env)?
	})
}

/// The `~` function.
pub fn NEG<I: IntType>() -> Function<I> {
	function!("~", env, |arg| {
		let ran = arg.run(env)?;

		#[cfg(feature = "iffy-extensions")]
		if env.flags().extensions.iffy.negating_a_list_inverts_it {
			if let Value::List(list) = ran {
				return Ok(list.reverse().into());
			}
		}

		ran.to_integer(env)?.negate()?.into()
	})
}

/// The `+` function.
pub fn ADD<I: IntType>() -> Function<I> {
	function!("+", env, |lhs, rhs| {
		//
		lhs.run(env)?.add(&rhs.run(env)?, env)?
	})
}

/// The `-` function.
pub fn SUBTRACT<I: IntType>() -> Function<I> {
	function!("-", env, |lhs, rhs| {
		//
		lhs.run(env)?.subtract(&rhs.run(env)?, env)?
	})
}

/// The `*` function.
pub fn MULTIPLY<I: IntType>() -> Function<I> {
	function!("*", env, |lhs, rhs| {
		//
		lhs.run(env)?.multiply(&rhs.run(env)?, env)?
	})
}

/// The `/` function.
pub fn DIVIDE<I: IntType>() -> Function<I> {
	function!("/", env, |lhs, rhs| {
		//
		lhs.run(env)?.divide(&rhs.run(env)?, env)?
	})
}

/// The `%` function.
pub fn REMAINDER<I: IntType>() -> Function<I> {
	function!("%", env, |lhs, rhs| {
		//
		lhs.run(env)?.remainder(&rhs.run(env)?, env)?
	})
}

/// The `^` function.
pub fn POWER<I: IntType>() -> Function<I> {
	function!("^", env, |lhs, rhs| {
		//
		lhs.run(env)?.power(&rhs.run(env)?, env)?
	})
}

/// The `<` function.
pub fn LESS_THAN<I: IntType>() -> Function<I> {
	function!("<", env, |lhs, rhs| {
		(lhs.run(env)?.compare(&rhs.run(env)?, env)? == Ordering::Less).into()
	})
}

/// The `>` function.
pub fn GREATER_THAN<I: IntType>() -> Function<I> {
	function!(">", env, |lhs, rhs| {
		(lhs.run(env)?.compare(&rhs.run(env)?, env)? == Ordering::Greater).into()
	})
}

/// The `?` function.
pub fn EQUALS<I: IntType>() -> Function<I> {
	function!("?", env, |lhs, rhs| {
		//
		lhs.run(env)?.equals(&rhs.run(env)?, env)?.into()
	})
}

/// The `&` function.
pub fn AND<I: IntType>() -> Function<I> {
	function!("&", env, |lhs, rhs| {
		let condition = lhs.run(env)?;

		if condition.to_boolean(env)? {
			rhs.run(env)?
		} else {
			condition
		}
	})
}

/// The `|` function.
pub fn OR<I: IntType>() -> Function<I> {
	function!("|", env, |lhs, rhs| {
		let condition = lhs.run(env)?;

		if condition.to_boolean(env)? {
			condition
		} else {
			rhs.run(env)?
		}
	})
}

/// The `;` function.
pub fn THEN<I: IntType>() -> Function<I> {
	function!(";", env, |lhs, rhs| {
		lhs.run(env)?;
		rhs.run(env)?
	})
}

/// The `=` function.
pub fn ASSIGN<I: IntType>() -> Function<I> {
	function!("=", env, |variable, value| {
		let ran = value.run(env)?;
		variable.assign(ran.clone(), env)?;
		ran
	})
}

/// The `WHILE` function.
pub fn WHILE<I: IntType>() -> Function<I> {
	function!("WHILE", env, |condition, body| {
		while condition.run(env)?.to_boolean(env)? {
			body.run(env)?;
		}

		Value::Null
	})
}

/// The `IF` function.
pub fn IF<I: IntType>() -> Function<I> {
	function!("IF", env, |condition, iftrue, iffalse| {
		if condition.run(env)?.to_boolean(env)? {
			iftrue.run(env)?
		} else {
			iffalse.run(env)?
		}
	})
}

/// The `GET` function.
pub fn GET<I: IntType>() -> Function<I> {
	function!("GET", env, |source, start, length| {
		//
		source.run(env)?.get(&start.run(env)?, &length.run(env)?, env)?
	})
}

/// The `SET` function.
pub fn SET<I: IntType>() -> Function<I> {
	function!("SET", env, |source, start, length, replacement| {
		//
		source.run(env)?.set(&start.run(env)?, &length.run(env)?, replacement.run(env)?, env)?
	})
}

/// The `VALUE` extension function.
///
/// This takes a single argument, converts it to a [`Text`](crate::value::Text) and interprets it
/// as a variable name. Then, it looks up the last assigned value to that variable.
#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn VALUE<I: IntType>() -> Function<I> {
	function!("VALUE", env, |arg| {
		let name = arg.run(env)?.to_text(env)?;
		env.lookup(&name)?.into()
	})
}

#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn HANDLE<I: IntType>() -> Function<I> {
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
pub fn YEET<I: IntType>() -> Function<I> {
	function!("YEET", env, |errmsg| {
		let errmsg = errmsg.run(env)?.to_text(env)?.to_string();

		return Err(Error::Custom(errmsg.into()));

		#[allow(unreachable_code)]
		Value::Null
	})
}

/// The `USE` function.
#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn USE<I: IntType>() -> Function<I> {
	function!("USE", env, |arg| {
		let filename = arg.run(env)?.to_text(env)?;
		let contents = env.read_file(&filename)?;

		env.play(&contents)?
	})
}

/// The `EVAL` function.
#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn EVAL<I: IntType>() -> Function<I> {
	function!("EVAL", env, |val| {
		let code = val.run(env)?.to_text(env)?;
		env.play(&code)?
	})
}

/// The `` ` `` function.
#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn SYSTEM<I: IntType>() -> Function<I> {
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
pub fn XSRAND<I: IntType>() -> ExtensionFunction<I> {
	xfunction!("XSRAND", env, |arg| {
		let seed = arg.run(env)?.to_integer(env)?;
		env.srand(seed);
		Value::Null
	})
}

/// **Compiler extension**: REV
#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn XREVERSE<I: IntType>() -> ExtensionFunction<I> {
	xfunction!("XREVERSE", env, |arg| {
		match arg.run(env)? {
			Value::Text(_text) => {
				// text.chars().collect::<Vec<Character>>().into_iter().rev().collect::<Text>().into()
				todo!()
			}
			Value::List(list) => {
				let mut eles = list.iter().cloned().collect::<Vec<Value<I>>>();
				eles.reverse();
				List::new(eles, env.flags()).unwrap().into()
			}
			other => return Err(Error::TypeError(other.typename(), "XRANGE")),
		}
	})
}

#[cfg(feature = "extensions")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
pub fn XRANGE<I: IntType>() -> ExtensionFunction<I> {
	xfunction!("XRANGE", env, |start, stop| {
		match start.run(env)? {
			Value::Integer(start) => {
				let stop = stop.run(env)?.to_integer(env)?;

				match start <= stop {
					true => List::new(
						(i64::from(start)..i64::from(stop))
							.map(|x| Value::from(crate::value::Integer::try_from(x).unwrap()))
							.collect::<Vec<Value<I>>>(),
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
