/// A set of flags that can be toggled to change how the interpreter runs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Flags {
	/// Knight specification conformity flags.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	pub compliance: ComplianceFlags,

	/// Extension-related flags.
	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub exts: ExtensionFlags,
}

/// Flags related to catching undefined behaviour in Knight programs.
///
/// This interpreter will always properly run all well-formed Knight programs. It'll also catch
/// some forms of undefined behaviour (such as division by zero) by default. However, some forms of
/// undefined behaviour in Knight are too expensive/cumbersome to check for. These flags can be used
/// to toggle some of these checks on.
///
/// Note that that there are some compile-time flags that should be enabled if 100% compliance is to
/// be achieved; these are simply the flags that don't affect runtime performance too much. But, if
/// the `strict-compliance` feature is enabled, _all_ forms of undefined behaviour will be caught.
///
/// The default value for each of these is normally `false`. However, if the `strict-compliance`
/// feature is enabled, they will all instead default to `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "compliance")]
#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
pub struct ComplianceFlags {
	/// Ensure [`QUIT`](crate::function::QUIT)'s argument is within `0..=127`.
	pub check_quit_bounds: bool,

	/// Require source files to be exactly one expression.
	pub forbid_trailing_tokens: bool,

	/// Ensure variable names are compliant.
	///
	/// This means that all variable names (either in source code or through the [`VALUE`](
	/// FunctionFlags::value) extension) must be:
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

	/// Ensures that [`?`](crate::function::EQUALS) is not with a [`BLOCK`](crate::function::BLOCK)
	/// return type.
	pub check_equals_params: bool,

	/// Ensures that [`%`](crate::function::REMAINDER) and [`^`](crate::function::POWER) are called
	/// with valid arguments only.
	///
	/// More specifically, this only allows positive integers for both arguments to `%`, and
	/// nonnegative numbers for the exponent for `^`. Regardless of this toggle, division and modulo
	/// by zero are checked.
	pub check_integer_function_bounds: bool,
}

#[cfg(feature = "compliance")]
impl Default for ComplianceFlags {
	fn default() -> Self {
		const STRICT_COMPLIANCE: bool = cfg!(feature = "strict-compliance");

		Self {
			check_quit_bounds: STRICT_COMPLIANCE,
			forbid_trailing_tokens: STRICT_COMPLIANCE,
			verify_variable_names: STRICT_COMPLIANCE,
			check_call_arg: STRICT_COMPLIANCE,
			limit_rand_range: STRICT_COMPLIANCE,
			check_equals_params: STRICT_COMPLIANCE,
			check_integer_function_bounds: STRICT_COMPLIANCE,
		}
	}
}

/// Flags for extensions to the Knight interpreter.
///
/// Normally, the flags default to `false`. However, if the `all-extensions` feature is enabled,
/// all extension flags default to true.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
pub struct ExtensionFlags {
	/// Assign to types other than variables.
	pub assign_to: AssignToFlags,

	/// Enable butilin function.
	pub fns: FunctionFlags,

	/// Extensions to types.
	pub tys: TypeFlags,

	/// Indexing either [`GET`](crate::function::GET) or [`SET`](crate::function::SET) with a
	/// negative number is that many from the end.
	pub negative_indexing: bool,

	/// Enables the list literal syntax (`{ TRUE FALSE NULL }` desugars to `++, TRUE, FALSE ,NULL`).
	pub list_literal: bool,
}

#[cfg(feature = "extensions")]
impl Default for ExtensionFlags {
	/// Normally all extensions features are disabled. However, if `all-extensions` is enabled,
	/// they all default to true.
	fn default() -> Self {
		const ALL_EXTENSIONS: bool = cfg!(feature = "all-extensions");

		Self {
			assign_to: AssignToFlags::default(),
			fns: FunctionFlags::default(),
			tys: TypeFlags::default(),
			negative_indexing: ALL_EXTENSIONS,
			list_literal: ALL_EXTENSIONS,
		}
	}
}

/// Flags to enable builtin native functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
pub struct FunctionFlags {
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
impl Default for FunctionFlags {
	/// Normally all function flags features are disabled. However, if `all-extensions` is enabled,
	/// they all default to true.
	fn default() -> Self {
		const ALL_EXTENSIONS: bool = cfg!(feature = "all-extensions");

		Self {
			value: ALL_EXTENSIONS,
			eval: ALL_EXTENSIONS,
			handle: ALL_EXTENSIONS,
			yeet: ALL_EXTENSIONS,
			r#use: ALL_EXTENSIONS,
			system: ALL_EXTENSIONS,
			xsrand: ALL_EXTENSIONS,
			xreverse: ALL_EXTENSIONS,
			xrange: ALL_EXTENSIONS,
		}
	}
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
pub struct TypeFlags {
	pub boolean: bool,
	pub list: bool,
	pub text: bool,
	pub integer: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
pub struct AssignToFlags {
	pub prompt: bool,
	pub system: bool,
	pub list: bool,
	pub text: bool,
}
