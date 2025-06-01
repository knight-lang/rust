use std::path::PathBuf;

use clap::{arg, command, value_parser, Arg, ArgAction, Args, Command, CommandFactory, Parser};
use knightrs_bytecode::{strings::Encoding, Options};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
	#[arg(short, long)]
	expression: Option<String>,

	#[arg(short, long)]
	file: Option<String>,
	// .next_help_heading(heading)

	/***************************************************************************
	 *                                Debugger                                 *
	 ***************************************************************************/
	/// Enable all debugger features
	#[arg(short, long, overrides_with = "_no_debugger")]
	debugger: bool,
	/// Undoes debugger
	#[arg(long, hide_short_help = true)]
	_no_debugger: bool, // underscore because nothing checks for it

	/// Print out stacktraces
	#[arg(long, hide_short_help = true, overrides_with = "no_stacktrace")]
	stacktrace: bool,
	/// Undoes stacktrace
	#[arg(long, hide_short_help = true)]
	no_stacktrace: bool,

	/// Ensure variables are always assigned
	#[arg(long, hide_short_help = true, overrides_with = "no_check_variables")]
	check_variables: bool,
	/// Undoes check_variables
	#[arg(long, hide_short_help = true)]
	no_check_variables: bool,

	/// Ensure parens are always balanced
	#[arg(long, hide_short_help = true, overrides_with = "no_check_parens")]
	check_parens: bool,
	/// Undoes check_parens
	#[arg(long, hide_short_help = true)]
	no_check_parens: bool,

	/***************************************************************************
	 *                                Embedded                                 *
	 ***************************************************************************/
	/// Enable all "embedded" features
	#[arg(long)]
	embedded: bool,

	/// Should exit when quitting the app
	#[arg(long, hide_short_help = true)]
	exit_when_quit: bool,

	/***************************************************************************
	 *                               Compliance                                *
	 ***************************************************************************/
	/// Enable all compliance features
	///
	/// Unlike --strict-compliance, this does not disable extensions
	#[arg(short = 'c', long, overrides_with = "_no_compliance")]
	compliance: bool,
	/// Undoes a previous --compliance
	#[arg(long, hide_short_help = true)]
	_no_compliance: bool, // underscore because nothing checks for it

	/// Like --compliance, except non-compliant flags are ignored
	///
	/// Unlike --strict-compliance, this does not disable extensions
	#[arg(short = 'C', long, overrides_with = "_no_strict_compliance")]
	strict_compliance: bool,
	/// Undoes a previous --strict_compliance
	#[arg(long, hide_short_help = true)]
	_no_strict_compliance: bool, // underscore because nothing checks for it

	/// Ensure containers are at most INTMAX elements long.
	#[arg(long, hide_short_help = true, overrides_with = "no_check_container_len")]
	check_container_len: bool,
	/// Undoes check_container_len
	#[arg(long, hide_short_help = true)]
	no_check_container_len: bool,

	/// Constrain integers to 32 bits
	#[arg(
		short = '3',
		long = "32-bit-int",
		hide_short_help = true,
		overrides_with = "no_i32_bit_ints"
	)]
	i32_bit_ints: bool,
	// Undoes i32_integers
	#[arg(long = "no-32-bit-int", hide_short_help = true)]
	no_i32_bit_ints: bool,

	/// Check for overflow in arithmetic operations
	#[arg(long, hide_short_help = true, overrides_with = "no_check_overflow")]
	check_overflow: bool,
	// Undoes check_overflow
	#[arg(long, hide_short_help = true)]
	no_check_overflow: bool,

	/// Check function bounds for integer functions.
	// TODO: maybe have this, along with `check_overflow` be function-specific?
	#[arg(long, hide_short_help = true, overrides_with = "no_check_int_fn_bounds")]
	check_int_fn_bounds: bool,
	// Undoes check_int_fn_bounds
	#[arg(long, hide_short_help = true)]
	no_check_int_fn_bounds: bool,

	/// Ensure variables are at most 127 chars long
	#[arg(long, hide_short_help = true, overrides_with = "no_validate_variable_name_len")]
	validate_variable_name_len: bool,
	// Undoes validate_variable_name_len
	#[arg(long, hide_short_help = true)]
	no_validate_variable_name_len: bool,

	/// Ensure at most 65535 variables are used
	#[arg(long, hide_short_help = true, overrides_with = "no_validate_variable_count")]
	validate_variable_count: bool,
	// Undoes validate_variable_count
	#[arg(long, hide_short_help = true)]
	no_validate_variable_count: bool,

	/// Require programs to be exactly one token long.
	///
	/// Without this option, trailing tokens are ignored
	#[arg(long, hide_short_help = true, overrides_with = "no_forbid_trailing_tokens")]
	forbid_trailing_tokens: bool,
	// Undoes forbid_trailing_tokens
	#[arg(long, hide_short_help = true)]
	no_forbid_trailing_tokens: bool,

	/// Only support BLOCKs in functions that the spec permits.
	///
	/// Without this, a handful of functions (such as `?` and `DUMP`) support them.
	#[arg(long, hide_short_help = true, overrides_with = "no_strict_blocks")]
	strict_blocks: bool,
	// Undoes strict_blocks
	#[arg(long, hide_short_help = true)]
	no_strict_blocks: bool,

	/// Don't allow blocks to be converted to other types
	#[arg(long, hide_short_help = true, overrides_with = "no_no_block_conversions")]
	no_block_conversions: bool,
	// Undoes no_block_conversions
	#[arg(long, hide_short_help = true)]
	no_no_block_conversions: bool,

	/// Limit `RANDOM` to be only from 0 to 32767
	#[arg(long, hide_short_help = true, overrides_with = "no_limit_random_range")]
	limit_random_range: bool,
	// Undoes limit_random_range
	#[arg(long, hide_short_help = true)]
	no_limit_random_range: bool,

	/// Require `QUIT` to be called with ints from 0 to 127.
	#[arg(long, hide_short_help = true, overrides_with = "no_check_quit_status_code")]
	check_quit_status_code: bool,
	// Undoes check_quit_status_code
	#[arg(long, hide_short_help = true)]
	no_check_quit_status_code: bool,

	/// Forbid some conversions (such as boolean -> list) that are allowed as an extension.
	#[arg(long, hide_short_help = true, overrides_with = "no_strict_conversions")]
	strict_conversions: bool,
	// Undoes strict_conversions
	#[arg(long, hide_short_help = true)]
	no_strict_conversions: bool,

	/***************************************************************************
	 *                               Extensions                                *
	 ***************************************************************************/
	/// Enables all extensions
	#[arg(short = 'E', long, overrides_with = "_no_extensions")]
	extensions: bool,
	/// Undoes a previous --extensions
	#[arg(long, hide_short_help = true)]
	_no_extensions: bool, // underscore because nothing checks for it

	/// Enable all extension types. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true)]
	ext_types: bool,

	/// Enable floats. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true)]
	ext_float: bool,

	/// Enable hashmaps. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true)]
	ext_hashmap: bool,

	/// Enable classes. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true)]
	ext_classes: bool,

	/// Enable negative indexing
	#[arg(long, hide_short_help = true)]
	ext_neg_indexing: bool,

	/// Enable the extension functions. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true)]
	ext_fns: bool,

	/// Enable the EVAL function.
	#[arg(long, hide_short_help = true)]
	ext_eval: bool,

	/// Enable the VALUE function.
	#[arg(long, hide_short_help = true)]
	ext_value: bool,

	/// Add support for the `_argv` variable, which is additional arguments on the cli.
	#[arg(long, hide_short_help = true)]
	ext_argv: bool,
	//     --[no-]ext-breaking-changes
	//     --[no-]ext-breaking-changes-negate-rev-collection
	//     --[no-]ext-breaking-changes-rand-can-be-negative
	//     --[no-]ext-list-literal          not implemented
	//     --[no-]ext-string-interopolation not implemented
	//     --[no-]ext-control-flow          XBREAK,XCONTINUE, XRETURN; partially working
	//     --[no-]ext-builtin-fns-boolean
	//     --[no-]ext-builtin-fns-string
	//     --[no-]ext-builtin-fns-list
	//     --[no-]ext-builtin-fns-integer
	//     --[no-]ext-builtin-fns-null
	//     --[no-]ext-builtin-fns-assign-to-strings
	//     --[no-]ext-builtin-fns-assign-to-random
}

impl Cli {
	pub fn options(&self) -> Result<Options, clap::Error> {
		let mut opts = Options::default();

		macro_rules! check_option {
			(
				feature = $feature:literal, default = $default:expr;
				$(opts.$($target:ident).+ = $yes:ident, $no:ident;)*
			) => {
				let default = $default;

				#[cfg(not(feature = $feature))]
				if default $(|| self.$yes)* {
					return Err(clap::Error::raw(
						clap::error::ErrorKind::ArgumentConflict,
						concat!("feature ", $feature, " is not enabled!")
					));
				}

				#[cfg(feature = $feature)]
				{
					$(
						opts.$($target).+ = if self.$yes {
							true
						} else if self.$no {
							false
						} else {
							default
						};
					)*
				}
			};
		}

		check_option! {
			feature = "debugger", default = self.debugger;

			opts.debugger.stacktrace = stacktrace, no_stacktrace;
			opts.check_parens = check_parens, no_check_parens;
			opts.check_variables = check_variables, no_check_variables;
		}

		check_option! {
			feature = "embedded", default = self.embedded;
			// opts.embedded.dont_exit_when_quitting = dont_exit_when_quitting, no_dont_exit_when_quitting;
		}

		check_option! {
			feature = "compliance", default = self.compliance || self.strict_compliance;

			opts.compliance.i32_integer = i32_bit_ints, no_i32_bit_ints;
			opts.compliance.check_overflow = check_overflow, no_check_overflow;
			opts.compliance.check_integer_function_bounds = check_int_fn_bounds, no_check_int_fn_bounds;
			opts.compliance.variable_name_length = validate_variable_name_len, no_validate_variable_name_len;
			opts.compliance.variable_count = validate_variable_count, no_validate_variable_count;
			opts.compliance.forbid_trailing_tokens = forbid_trailing_tokens, no_forbid_trailing_tokens;
			opts.compliance.strict_blocks = strict_blocks, no_strict_blocks;
			opts.compliance.no_block_conversions = no_block_conversions, no_no_block_conversions;
			opts.compliance.limit_rand_range = limit_random_range, no_limit_random_range;
			opts.compliance.check_quit_status_codes = check_quit_status_code, no_check_quit_status_code;
			opts.compliance.strict_conversions = strict_conversions, no_strict_conversions;
		}

		if !self.strict_compliance {
			check_option! {
				feature = "extensions", default = self.extensions;
			}
		}

		// TODO: extensions

		Ok(opts)
	}
}

pub fn get_options() -> (Options, String) {
	let cli = Cli::parse();

	let opts = match cli.options() {
		Ok(opts) => opts,
		Err(err) => err.format(&mut Cli::command()).exit(),
	};

	(opts, cli.expression.expect("for now, -e is required"))
}
