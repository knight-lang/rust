# Knight v2.0 in Rust
This 
This is the Rust version of Knight. It's intended to be usable within embedded applications.

More details to come later, once I finish documentation.


# Knight v2.0: Rust Edition 
This is a fully functioning [Knight 2.0](https://github.com/knight-lang/knight-lang) implementation in Rust. More details about Knight, its license, and specifications can be found in the [knight-lang](https://github.com/knight-lang/knight-lang) repo.

The Rust implementation is a "paragon" implementation: It implements nearly all the suggested extensions—and some compiler-specific quality of life ones—, is able to be embedded within other Rust programs (eg the `discord-bot` directory), and even catches _all_ undefined behaviour.

# Usage
Simply run `cargo run -- (-e 'expr' | -f filename)`, and it'll run your program. Alternatively, you can instead compile the binary with `cargo build`, and then execute it via `./target/debug/knight (-e 'expr' | -f filename)`.

# Enabling strict compliance
By default, the "normal" extensions are enabled, and only simple forms of undefined behaviour are caught. However, you can use `cargo run --no-default-features --features=strict-compliance -- ...` to disable all extensions, and catch _every single form of undefined behaviour_. This can be somewhat slow, however.

# Compiler Flags
The following is the list of flags, and their descriptions. To enable flags, pass `--features=<flag1>,<flag2>,...` as an argument after the `build`/`run`. Note that most of these flags are enabled by default (see the `Cargo.toml`'s `features.default` value for specifics); to disable them, you must run `--no-default-features`, which will only use features you explicitly state

## Disabling Optimizations
- `multithreaded`: Enable this to turn on multithreading support. Since Knight is normally single threaded, the Rusut implementation by default uses single-threaded data structures such as `Rc` and `RefCell`. 

## Spec-defined Extensions
- **`spec-defined-extensions`**: Enables everything in this section.
- `value-function`: Enables the `VALUE` function
- `eval-function`: Enables the `EVAL` function
- `assign-to-anything`: Convert strings to variables if theyre the first arg to `_`.
- `handle-function`: Enables the use of `HANDLE`, which is try-catch.
- `yeet-function`: Enables the use of `YEET`, which is throw.
- `use-function`: Enables the `USE` function, which is for importing files.
- `system-function`: Enables the `$` function, which is for doing system calls.

## Compiler Extensions
- **list-extensions**: Provides useful extensions for lists, such as `* list BLOCK ...` will map all the elements of the list to the return value of the block.
- **string-extensions**: Provides useful extensions for strings, such as `/ string sep` will split a string by the `sep`.
negative-ranges = []
assign-to-lists = []

extension-functions = [] # Required for all `X` functions.


# Enables `XSRAND`
xsrand-function = ["extension-functions"]
xrange-function = ["extension-functions"]
xreverse-function = ["extension-functions"]

# Enable most extensions
normal-extensions = [
	"value-function",
	"assign-to-anything",
	"handle-function",
	"use-function",
	# "xsrand-function",
]

all-extensions = ["normal-extensions", "qol-extensions"]

#######################################
##     Quality-of-life extensions    ##
#######################################
# All quality of life extensions may be removed in the future without it
# being considered a breaking change; these are just for golfing convenience.

# Causes `GET` and `SET` to return empty strings on errors, not errors.
no-oob-errors = []

# Allows you to use negative indexes in `GET` and `SET` 
negative-indexing = []
- `negative-ranges`:

# Let's you specify what the next `PROMPT` should return.
#
# If this is set along with `assign-to-anything`, this will take precedence;
# use `+""P` if you want to assign to `PROMPT`'s return value.
assign-to-prompt = []
assign-to-system = ["system-function"]

assignment-overloads = ["assign-to-prompt", "assign-to-system"]

# Enable all quality-of-life extensions
qol-extensions = ["no-oob-errors", "negative-indexing", "assign-to-prompt"]

# split-strings = ["arrays"]
# string-formatting = ["arrays"]

#######################################
##          Spec Compliance          ##
#######################################

container-length-limit = []
forbid-trailing-tokens = []

# Restrict source files and strings to the bytes defined in the Knight spec.
strict-charset = [] # TODO: make this work

# Catch undefined integer arithmetic
checked-overflow = []

# Use `i32` for integers, which is the minimum required range. Also enables some niche things.
strict-integers = []

# Ensure BLOCK return values are handled correctly.
strict-block-return-value = []

# Ensure that all variable names are valid (eg `VALUE 1` would fail.)
verify-variable-names = []

# Exactly follow knight specs.
strict-compliance = [
	"strict-integers",
	"checked-overflow",
	"forbid-trailing-tokens",
	"strict-charset",
	"strict-block-return-value",
	"verify-variable-names",
	"container-length-limit",
]

[dependencies]
rand = "0.8"
once_cell = "1.7"
lazy_static = "1.4"
cfg-if = "1.0"
clap = { version = "2.33", optional = true }
tap = "1.0"
static_assertions = "1.1"

[package.metadata.docs.rs]
rustdoc-args = ["--cfg", "doc_cfg"]

[[bin]]
name = "knight"
path = "src/main.rs"
# required-features = ["clap"]


<!-- 
# Compiler-specific addons
A couple of custom extensions were added in addition to those mentioned by the spec:
- `assign-to-prompt`: If you assign to the `PROMPT` function (i.e. `= PROMPT ...`), then the next time `PROMPT` is called, that value will be returned.
- `assign-to-system`: The same as `assign-to-prompt`, except it's for the `$` function
- 



#######################################
##     Quality-of-life extensions    ##
#######################################
# All quality of life extensions may be removed in the future without it
# being considered a breaking change; these are just for golfing convenience.

# Causes `GET` and `SET` to return empty strings on errors, not errors.
no-oob-errors = []

# Allows you to use negative indexes in `GET` and `SET` 
negative-indexing = []

# Let's you specify what the next `PROMPT` should return.
#
# If this is set along with `assign-to-anything`, this will take precedence;
# use `+""P` if you want to assign to `PROMPT`'s return value.
assign-to-prompt = []
assign-to-system = ["system-function"]

assignment-overloads = ["assign-to-prompt", "assign-to-system"]
 -->
