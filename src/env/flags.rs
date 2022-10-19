//! Flags to change how the Knight interpreter works at runtime.

/// A set of flags that can be toggled to change how the interpreter runs.
///
/// Normally, all flags default to `false`. However, if `strict-compliance` is enabled, then the
/// compliance flags will default to `true`. Likewise, is `all-extensions` is enabled, the extension
/// flags will default to `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
#[cfg_attr(feature = "clap", command(next_line_help = false))]
#[non_exhaustive]
pub struct Flags {
	/// Knight specification conformity flags.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	#[cfg_attr(feature = "clap", command(flatten))]
	pub compliance: Compliance,

	/// Extension-related flags.
	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	#[cfg_attr(feature = "clap", command(flatten))]
	pub extensions: Extensions,
}

impl Default for Flags {
	#[inline]
	fn default() -> Self {
		DEFAULT
	}
}

impl Default for &Flags {
	#[inline]
	fn default() -> Self {
		&DEFAULT
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
			negating_a_list_inverts_it: cfg!(feature = "all-iffy-extensions"),
			unassigned_variables_default_to_null: cfg!(feature = "all-iffy-extensions"),
			negative_random_integers: cfg!(feature = "all-iffy-extensions"),
		},
		negative_indexing: ALL_EXTENSIONS,
		list_literal: ALL_EXTENSIONS,
	},
};

cfg_if! {
if #[cfg(feature = "compliance")] {
	/// Flags related to catching undefined behaviour in Knight programs.
	///
	/// This interpreter will always properly run all well-formed Knight programs. It'll also catch
	/// some forms of undefined behaviour (such as division by zero) by default. However, some forms
	/// of undefined behaviour in Knight are too expensive/cumbersome to check for. These flags can
	/// be used to toggle some of these checks on.
	///
	/// While these flags will catch most undefined behaviour, to catch _all_ forms, the [`Value`](
	/// crate::value::Value)'s generics should be set to [`Wrapping<i32>`](
	/// crate::value::integer::Wrapping) and [`KnightEncoding`](crate::value::text::KnightEncoding).
	///
	/// The default value for each of these is normally `false`. However, if the `strict-compliance`
	/// feature is enabled, they will all instead default to `true`.
	#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	#[cfg_attr(feature = "clap", derive(clap::Args))]
	#[cfg_attr(feature = "clap", command(next_line_help = false))]
	#[non_exhaustive]
	pub struct Compliance {
		/// Ensure [`QUIT`](crate::function::QUIT)'s argument is within `0..=127`.
		#[cfg_attr(feature = "clap", arg(long))]
		pub check_quit_bounds: bool,

		/// Require source files to be exactly one expression.
		#[cfg_attr(feature = "clap", arg(long))]
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
		#[cfg_attr(feature = "clap", arg(long))]
		pub verify_variable_names: bool,

		/// Ensure that [`CALL`](crate::function::CALL) is only ever run on Blocks.
		#[cfg_attr(feature = "clap", arg(long))]
		pub check_call_arg: bool,

		/// Limit [`RANDOM`](crate::function::RANDOM) to return values to `0..=0x7fff`.
		///
		/// Without this, it'll return a value from `0..=<max size of integer>`.
		#[cfg_attr(feature = "clap", arg(long))]
		pub limit_rand_range: bool,

		/// Ensures that [`?`](crate::function::EQUALS) is not called with with a [`BLOCK`](
		/// crate::function::BLOCK)'s return value.
		#[cfg_attr(feature = "clap", arg(long))]
		pub check_equals_params: bool,

		/// Ensures that the length of [`Text`](crate::value::Text)s and [`List`](crate::value::List)s
		/// are no larger than [`i32::MAX`].
		#[cfg_attr(feature = "clap", arg(long))]
		pub check_container_length: bool,

		/// Ensures that [`Integer::power`](crate::value::Integer::power) and [`Integer::remainder`](
		/// crate::value::Integer::remainder) and are called with valid arguments only.
		///
		/// More specifically, this only allows positive integers for both arguments to `remainder`,
		/// and nonnegative numbers for the exponent for `power`. Regardless of this flag, division
		/// and modulo by zero are checked.
		#[cfg_attr(feature = "clap", arg(long))]
		pub check_integer_function_bounds: bool,
	}

	impl Default for Compliance {
		#[inline]
		fn default() -> Self {
			DEFAULT.compliance
		}
	}
}} // compliance

cfg_if! {
if #[cfg(feature = "extensions")] {
	/// Flags for extensions to the Knight interpreter.
	#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	#[cfg_attr(feature = "clap", derive(clap::Args))]
	#[cfg_attr(feature = "clap", command(next_line_help = false))]
	#[non_exhaustive]
	pub struct Extensions {
		/// Extension function flags.
		#[cfg_attr(feature = "clap", command(flatten))]
		pub functions: Functions,

		/// Extensions to types.
		#[cfg_attr(feature = "clap", command(flatten))]
		pub types: Types,

		/// Assign to types other than variables.
		#[cfg_attr(feature = "clap", command(flatten))]
		pub assign_to: AssignTo,

		/// Things that change how knight works, or are otherwise iffy.
		#[cfg(feature = "iffy-extensions")]
		#[cfg_attr(docsrs, doc(cfg(feature = "iffy-extensions")))]
		#[cfg_attr(feature = "clap", command(flatten))]
		pub iffy: Iffy,

		/// Indexing either [`GET`](crate::function::GET) or [`SET`](crate::function::SET) with a
		/// negative number is that many from the end.
		#[cfg_attr(feature = "clap", arg(long))]
		pub negative_indexing: bool,

		/// Enables the list literal syntax
		///
		/// For example, `{ TRUE FALSE NULL }` desugars to `++, TRUE, FALSE ,NULL`.
		#[cfg_attr(feature = "clap", arg(long))]
		pub list_literal: bool,
	}

	impl Default for Extensions {
		#[inline]
		fn default() -> Self {
			DEFAULT.extensions
		}
	}

	/// Flags to enable extension functions.
	#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	#[cfg_attr(feature = "clap", derive(clap::Args))]
	#[cfg_attr(feature = "clap", command(next_line_help = false))]
	#[non_exhaustive]
	pub struct Functions {
		/// Enables the [`VALUE`](crate::function::VALUE) function.
		#[cfg_attr(feature = "clap", arg(long))]
		pub value: bool,

		/// Enables the [`EVAL`](crate::function::EVAL) function.
		#[cfg_attr(feature = "clap", arg(long))]
		pub eval: bool,

		/// Enables the [`HANDLE`](crate::function::HANDLE) function.
		#[cfg_attr(feature = "clap", arg(long))]
		pub handle: bool,

		/// Enables the [`YEET`](crate::function::YEET) function.
		#[cfg_attr(feature = "clap", arg(long))]
		pub yeet: bool,

		/// Enables the [`USE`](crate::function::USE) function.
		#[cfg_attr(feature = "clap", arg(long))]
		pub r#use: bool,

		/// Enables the [`$`](crate::function::SYSTEM) function.
		#[cfg_attr(feature = "clap", arg(long))]
		pub system: bool,

		/// Enables the [`XSRAND`](crate::function::XSRAND) function.
		#[cfg_attr(feature = "clap", arg(long))]
		pub xsrand: bool,

		/// Enables the [`XREVERSE`](crate::function::XREVERSE) function.
		#[cfg_attr(feature = "clap", arg(long))]
		pub xreverse: bool,

		/// Enables the [`XRANGE`](crate::function::XRANGE) function.
		#[cfg_attr(feature = "clap", arg(long))]
		pub xrange: bool,
	}

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
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	#[cfg_attr(feature = "clap", derive(clap::Args))]
	#[cfg_attr(feature = "clap", command(next_line_help = false))]
	#[non_exhaustive]
	pub struct Types {
		/// Enables [`Boolean`](crate::value::Boolean)-related extensions.
		///
		/// - If a boolean is passed to `+`, it converts the second argument to a boolean and returns
		///   whether either argument is truthy.
		/// - If a boolean is passed to `*`, it converts the second argument to a boolean and returns
		///   whether both arguments are truthy.
		#[cfg_attr(feature = "clap", arg(long))]
		pub boolean: bool,

		/// Enables [`Integer`](crate::value::Integer)-related extensions.
		///
		/// - If an integer is passed to `[`, it'll return the most significant digit. If the integer
		///   is negative, the resulting value will be negative.
		/// - If an integer is passed to `]`, it'll return everything but the most significant digit.
		///   If the integer is negative, the resulting value will be negative.
		#[cfg_attr(feature = "clap", arg(long))]
		pub integer: bool,

		/// Enables [`List`](crate::value::List)-related extensions.
		///
		/// - If a list is passed to `-`, it converts the second argument to a list and return a new
		///   list comprised of elements only in the first list. Duplicate elements are also removed.
		/// - If a list is passed to `*` and the second argument is, after being run, a [block], then
		///   the list is [mapped](crate::value::List::map).
		/// - If a list is passed to `/` and the second argument is, after being run, a [block], then
		///   the list is [reduced](crate::value::List::reduce).
		/// - If a list is passed to `%` and the second argument is, after being run, a [block], then
		///   the list is [filtered](crate::value::List::filter).
		///
		/// [block]: crate::Ast
		#[cfg_attr(feature = "clap", arg(long))]
		pub list: bool,

		/// Enables [`Text`](crate::value::Text)-related extensions.
		///
		/// - If a text is passed to `-`, it converts the second argument to a text and then [removes
		///   that substring](crate::value::Text::remove_substr)
		/// - If a text is passed to `/`, it converts the second to a text and then [splits the first
		///   by the second](crate::value::Text::split).
		#[cfg_attr(feature = "clap", arg(long))]
		pub text: bool,
	}

	impl Default for Types {
		#[inline]
		fn default() -> Self {
			DEFAULT.extensions.types
		}
	}

	/// Flags related to assigning to non-[`Variable`](crate::env::Variable) types.
	#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	#[cfg_attr(feature = "clap", derive(clap::Args))]
	#[cfg_attr(feature = "clap", command(next_line_help = false))]
	#[non_exhaustive]
	pub struct AssignTo {
		/// Allows you to assign to `PROMPT`. See [`Prompt`](crate::env::Prompt) for details.
		#[cfg_attr(feature = "clap", arg(long))]
		pub prompt: bool,

		/// Allows you to assign to `OUTPUT`. See [`Output`](crate::env::Output) for details.
		#[cfg_attr(feature = "clap", arg(long))]
		pub output: bool,

		/// Allows you to assign to `$`. See [`System`](crate::env::System) for details.
		#[cfg_attr(feature = "clap", arg(long="assign-to-system", name="assign-to-system"))]
		pub system: bool,

		/// Allows you to assign to [`List`](crate::value::List)s. This in essence is destructuring.
		///
		/// # Example
		/// ```knight
		/// ; = +@abc +@1 2 3
		/// : OUTPUT * (-a b) c #=> -3
		/// ```
		#[cfg_attr(feature = "clap", arg(long="assign-to-list", name="assign-to-list"))]
		pub list: bool,

		/// Allows you to assign to [`Text`](crate::value::Text)s. This lets you dynamically assign to
		/// variables.
		///
		/// # Example
		/// ```knight
		/// ; = (+"hi" 1) 4
		/// : OUTPUT hi1 #=> 4
		/// ```
		#[cfg_attr(feature = "clap", arg(long="assign-to-text", name="assign-to-text"))]
		pub text: bool,
	}

	impl Default for AssignTo {
		#[inline]
		fn default() -> Self {
			DEFAULT.extensions.assign_to
		}
	}
}}

cfg_if! {
if #[cfg(feature = "iffy-extensions")] {
	/// Features that change how vanilla Knight is interpreted, or are otherwise iffy.
	#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	#[cfg_attr(docsrs, doc(cfg(feature = "iffy-extensions")))]
	#[cfg_attr(feature = "clap", derive(clap::Args))]
	#[cfg_attr(feature = "clap", command(next_line_help = false))]
	#[non_exhaustive]
	pub struct Iffy {
		/// Instead of erroring on undefined variables, have them default to `NULL`
		#[cfg_attr(feature = "clap", arg(long))]
		pub unassigned_variables_default_to_null: bool,

		/// `~list` reverses it, not gets its length and negates it.
		#[cfg_attr(feature = "clap", arg(long))]
		pub negating_a_list_inverts_it: bool,

		/// `RANDOM` can return negative integers
		#[cfg_attr(feature = "clap", arg(long))]
		pub negative_random_integers: bool,
	}

	impl Default for Iffy {
		#[inline]
		fn default() -> Self {
			DEFAULT.extensions.iffy
		}
	}
}}
