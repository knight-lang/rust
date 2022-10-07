#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Flags {
	pub assign_to: AssignToFlags,
	pub compliance: ComplianceFlags,
	pub negative_indexing: bool,
	pub exts: ExtensionFlags,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComplianceFlags {
	pub check_quit_bounds: bool,
	pub forbid_trailing_tokens: bool,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtensionFlags {
	pub ascii_on_lists: bool,
	pub boolean: bool,
	pub list: bool,
	pub text: bool,
	pub list_literal: bool,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssignToFlags {
	pub prompt: bool,
	pub system: bool,
	pub list: bool,
	pub text: bool,
}

/*

#######################################
##      Spec-defined Extensions      ##
#######################################
value-function    = [] # Enables the `VALUE` function
eval-function     = [] # Enables the `EVAL` function
assign-to-strings = [] # Convert strings to variables if they're the first arg to `=`.
handle-function   = [] # Enables the use of `HANDLE`, which is try-catch.
yeet-function     = [] # Enables the use of `YEET`, which is throw.
use-function      = [] # Enables the `USE` function, which is for importing files.
system-function   = [] # Enables the `$` function, which is for doing system calls.
spec-extensions   = [  # Enables everything in this section.
	"value-function",
	"eval-function",
	"assign-to-strings",
	"handle-function",
	"yeet-function",
	"use-function",
	"system-function",
]

#######################################
##        Compiler Extensions        ##
#######################################
list-extensions     = [] # Enables useful extensions for lists, such as `* list BLOCK ...` is "map".
string-extensions   = [] # Enables useful extensions for strings, such as `/ string sep` is "split".
xsrand-function     = [] # Enables the use of `XSRAND`, used for seeding `RANDOM`.
xrange-function     = [] # Enables the use of `XRANGE`, which is used to construct lists
xreverse-function   = [] # Enables the use of `XREVERSE`, which can be used to reverse lists.
compiler-extensions = [  # Enables everything in this section
	"list-extensions",
	"string-extensions",
	"xsrand-function",
	"xrange-function",
	"xreverse-function",
]

#######################################
##     Quality-Of-Life Extensions    ##
#######################################
negative-index-length = [] # Use negative start index in `GET`/`SET` for lists/strings.
assign-to-prompt      = [] # Assign to `PROMPT` changes what it returns next, in a queue.
assign-to-system      = ["system-function"] # Assign to `$` changes its next value, in a queue.
assign-to-lists       = [] # Assignment to lists enables destructoring.
assignment-overloads  = [  # Enables all assignment overloads
	"assign-to-strings",
	"assign-to-prompt",
	"assign-to-system",
	"assign-to-lists",
]
qol-extensions        = [ # Enables everything in thiss section
	"negative-index-length",
	"assignment-overloads",
]

#######################################
##          Spec Compliance          ##
#######################################
forbid-trailing-tokens = [] # Only allow a single expression for a program.
container-length-limit = [] # Ensures that containers do not grow larger than `i32::MAX`.
strict-charset         = [] # Only support Knight's strict subset of ASCII.
checked-overflow       = [] # Checks for under/overflow in integer arithmetic operations.
strict-integers        = [] # Use `i32` for integers; also enables some niche checks.
strict-call-argument   = [] # Ensures that `CALL` is only ever executed with `BLOCK`'s return value.
verify-variable-names  = [] # Ensures that all variable names are at most 127 characters long.
strict-compliance      = [  # Enables everything in this section, as well as more niche things.
	"forbid-trailing-tokens",
	"container-length-limit",
	"strict-charset",
	"checked-overflow",
	"strict-integers",
	"strict-call-argument",
	"verify-variable-names",
]
*/
