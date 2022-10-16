//! Flags to change how the Knight interpreter works at runtime.

/// A set of flags that can be toggled to change how the interpreter runs.
///
/// Normally, all flags default to `false`. However, if `strict-compliance` is enabled, then the
/// compliance flags will default to `true`. Likewise, is `all-extensions` is enabled, the extension
/// flags will default to `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Flags {
	/// Knight specification conformity flags.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	pub compliance: Compliance,

	/// Extension-related flags.
	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub extensions: Extensions,
}

impl Default for Flags {
	#[inline]
	fn default() -> Self {
		DEFAULT
	}
}

#[cfg(feature = "compliance")]
const STRICT_COMPLIANCE: bool = cfg!(feature = "strict-compliance");

#[cfg(feature = "extensions")]
const ALL_EXTENSIONS: bool = cfg!(feature = "all-extensions");

// We need this as a `static` because `env::Builder::default` needs to take a reference to a Flag.
pub(crate) static DEFAULT: Flags = Flags {
	#[cfg(feature = "compliance")]
	compliance: Compliance {
		check_quit_bounds: STRICT_COMPLIANCE,
		forbid_trailing_tokens: STRICT_COMPLIANCE,
		verify_variable_names: STRICT_COMPLIANCE,
		check_call_arg: STRICT_COMPLIANCE,
		limit_rand_range: STRICT_COMPLIANCE,
		check_equals_params: STRICT_COMPLIANCE,
		check_container_length: STRICT_COMPLIANCE,
		check_integer_function_bounds: STRICT_COMPLIANCE,
	},
	#[cfg(feature = "extensions")]
	extensions: Extensions {
		assign_to: AssignTo {
			prompt: ALL_EXTENSIONS,
			system: ALL_EXTENSIONS,
			output: ALL_EXTENSIONS,
			list: ALL_EXTENSIONS,
			text: ALL_EXTENSIONS,
		},
		functions: Functions {
			value: ALL_EXTENSIONS,
			eval: ALL_EXTENSIONS,
			handle: ALL_EXTENSIONS,
			yeet: ALL_EXTENSIONS,
			r#use: ALL_EXTENSIONS,
			system: ALL_EXTENSIONS,
			xsrand: ALL_EXTENSIONS,
			xreverse: ALL_EXTENSIONS,
			xrange: ALL_EXTENSIONS,
		},
		types: Types {
			boolean: ALL_EXTENSIONS,
			list: ALL_EXTENSIONS,
			text: ALL_EXTENSIONS,
			integer: ALL_EXTENSIONS,
		},
		#[cfg(feature = "iffy-extensions")]
		iffy: Iffy {
			negating_a_list_inverts_it: cfg!(feature = "iffy-extensions"),
			unassigned_variables_default_to_null: cfg!(feature = "iffy-extensions"),
			negative_random_integers: cfg!(feature = "iffy-extensions"),
		},
		negative_indexing: ALL_EXTENSIONS,
		list_literal: ALL_EXTENSIONS,
	},
};

/// Flags related to catching undefined behaviour in Knight programs.
///
/// This interpreter will always properly run all well-formed Knight programs. It'll also catch
/// some forms of undefined behaviour (such as division by zero) by default. However, some forms of
/// undefined behaviour in Knight are too expensive/cumbersome to check for. These flags can be used
/// to toggle some of these checks on.
///
/// While these flags will catch most undefined behaviour, to catch _all_ forms, the [`Value`](
/// crate::value::Value)'s generics should be set to [`Wrapping<i32>`](
/// crate::value::integer::Wrapping) and [`KnightEncoding`](crate::value::text::KnightEncoding).
///
/// The default value for each of these is normally `false`. However, if the `strict-compliance`
/// feature is enabled, they will all instead default to `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "compliance")]
#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
pub struct Compliance {
	/// Ensure [`QUIT`](crate::function::QUIT)'s argument is within `0..=127`.
	pub check_quit_bounds: bool,

	/// Require source files to be exactly one expression.
	pub forbid_trailing_tokens: bool,

	/// Ensure variable names are compliant.
	///
	/// This means that all variable names (either in source code or through the [`VALUE`](
	/// Functions::value) extension) must be:
	///
	/// - nonempty,
	/// - less than 128 characters long
	/// - start with either `_` or a lower case ASCII character.
	/// - everything but the first character can be either `_`, a lower case ASCII character, or
	///   an ASCII digit.
	pub verify_variable_names: bool,

	/// Ensure that [`CALL`](crate::function::CALL) is only ever run on Blocks.
	pub check_call_arg: bool,

	/// Limit [`RANDOM`](crate::function::RANDOM) to return values to `0..=0x7fff`.
	///
	/// Without this, it'll return a value from `0..=<max size of integer>`.
	pub limit_rand_range: bool,

	/// Ensures that [`?`](crate::function::EQUALS) is not called with with a [`BLOCK`](
	/// crate::function::BLOCK)'s return value.
	pub check_equals_params: bool,

	/// Ensures that the length of [`Text`](crate::value::Text)s and [`List`](crate::value::List)s
	/// are no larger than [`i32::MAX`].
	pub check_container_length: bool,

	/// Ensures that [`Integer::remainder`](crate::value::Integer::remainder) and [`Integer::power`](
	/// crate::value::Integer::power) are called with valid arguments only.
	///
	/// More specifically, this only allows positive integers for both arguments to `remainder`, and
	/// nonnegative numbers for the exponent for `power`. Regardless of this flag, division and
	/// modulo by zero are checked.
	pub check_integer_function_bounds: bool,
}

#[cfg(feature = "compliance")]
impl Default for Compliance {
	#[inline]
	fn default() -> Self {
		DEFAULT.compliance
	}
}

/// Flags for extensions to the Knight interpreter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
pub struct Extensions {
	/// Extension function flags.
	pub functions: Functions,

	/// Extensions to types.
	pub types: Types,

	/// Assign to types other than variables.
	pub assign_to: AssignTo,

	/// Things that change how knight works, or are otherwise iffy.
	#[cfg(feature = "iffy-extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "iffy-extensions")))]
	pub iffy: Iffy,

	/// Indexing either [`GET`](crate::function::GET) or [`SET`](crate::function::SET) with a
	/// negative number is that many from the end.
	pub negative_indexing: bool,

	/// Enables the list literal syntax (`{ TRUE FALSE NULL }` desugars to `++, TRUE, FALSE ,NULL`).
	pub list_literal: bool,
}

#[cfg(feature = "extensions")]
impl Default for Extensions {
	#[inline]
	fn default() -> Self {
		DEFAULT.extensions
	}
}

/// Flags to enable extension functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
pub struct Functions {
	/// Enables the [`VALUE`](crate::function::VALUE) function.
	pub value: bool,

	/// Enables the [`EVAL`](crate::function::EVAL) function.
	pub eval: bool,

	/// Enables the [`HANDLE`](crate::function::HANDLE) function.
	pub handle: bool,

	/// Enables the [`YEET`](crate::function::YEET) function.
	pub yeet: bool,

	/// Enables the [`USE`](crate::function::USE) function.
	pub r#use: bool,

	/// Enables the [`$`](crate::function::SYSTEM) function.
	pub system: bool,

	/// Enables the [`XSRAND`](crate::function::XSRAND) function.
	pub xsrand: bool,

	/// Enables the [`XREVERSE`](crate::function::XREVERSE) function.
	pub xreverse: bool,

	/// Enables the [`XRANGE`](crate::function::XRANGE) function.
	pub xrange: bool,
}

#[cfg(feature = "extensions")]
impl Default for Functions {
	#[inline]
	fn default() -> Self {
		DEFAULT.extensions.functions
	}
}

/// Flags to enable additional functionality for types.
///
/// See each flag for more details on what exactly it enables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
pub struct Types {
	/// Enables [`Boolean`](crate::value::Boolean)-related extensions.
	///
	/// - If a boolean is passed to `+`, it converts the second argument to a boolean and returns
	///   whether either argument is truthy.
	/// - If a boolean is passed to `*`, it converts the second argument to a boolean and returns
	///   whether both arguments are truthy.
	pub boolean: bool,

	/// Enables [`Integer`](crate::value::Integer)-related extensions.
	///
	/// - If an integer is passed to `[`, it'll return the most significant digit. If the integer is
	///   negative, the resulting value will be negative.
	/// - If an integer is passed to `]`, it'll return everything but the most significant digit.
	///   If the integer is negative, the resulting value will be negative.
	pub integer: bool,

	/// Enables [`List`](crate::value::List)-related extensions.
	///
	/// - If a list is passed to `-`, it converts the second argument to a list and return a new list
	///   comprised of elements only found in the first list. Duplicate elements are also removed.
	/// - If a list is passed to `*` and the second argument is, after being run, a [block], then the
	///   list is [mapped](crate::value::List::map).
	/// - If a list is passed to `/` and the second argument is, after being run, a [block], then the
	///   list is [reduced](crate::value::List::reduce).
	/// - If a list is passed to `%` and the second argument is, after being run, a [block], then the
	///   list is [filtered](crate::value::List::filter).
	///
	/// [block]: crate::Ast
	pub list: bool,

	/// Enables [`Text`](crate::value::Text)-related extensions.
	///
	/// - If a text is passed to `-`, it converts the second argument to a text and then [removes
	///   that substring](crate::value::Text::remove_substr)
	/// - If a text is passed to `/`, it converts the second to a text and then [splits the first by
	///   the second](crate::value::Text::split).
	pub text: bool,
}

#[cfg(feature = "extensions")]
impl Default for Types {
	#[inline]
	fn default() -> Self {
		DEFAULT.extensions.types
	}
}

/// Flags related to assigning to non-[`Variable`](crate::env::Variable) types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
pub struct AssignTo {
	/// Allows you to assign to `PROMPT`. See [`Prompt`](crate::env::Prompt) for details.
	pub prompt: bool,

	/// Allows you to assign to `OUTPUT`. See [`Output`](crate::env::Output) for details.
	pub output: bool,

	/// Allows you to assign to `$`. See [`System`](crate::env::System) for details.
	pub system: bool,

	/// Allows you to assign to [`List`](crate::value::List)s. This in essence is destructuring.
	///
	/// # Example
	/// ```knight
	/// ; = +@abc +@1 2 3
	/// : OUTPUT * (-a b) c #=> -3
	/// ```
	pub list: bool,

	/// Allows you to assign to [`Text`](crate::value::Text)s. This lets you dynamically assign to
	/// variables.
	///
	/// # Example
	/// ```knight
	/// ; = (+"hi" 1) 4
	/// : OUTPUT hi1 #=> 4
	/// ```
	pub text: bool,
}

#[cfg(feature = "extensions")]
impl Default for AssignTo {
	#[inline]
	fn default() -> Self {
		DEFAULT.extensions.assign_to
	}
}

/// Features that change how vanilla Knight is interpreted, or are otherwise iffy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "iffy-extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "iffy-extensions")))]
pub struct Iffy {
	/// Instead of erroring on undefined variables, have them default to `NULL`
	pub unassigned_variables_default_to_null: bool,

	/// `~list` reverses it, not gets its length and negates it.
	pub negating_a_list_inverts_it: bool,

	/// `RANDOM` can return negative integers
	pub negative_random_integers: bool,
}
