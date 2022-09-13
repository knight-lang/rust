# Knight v2.0 in Rust
This is the Rust version of Knight. It's intended to be usable as both a standalone binary and within embedded applications.

# Knight v2.0: Rust Edition 
This is a fully functioning [Knight 2.0](https://github.com/knight-lang/knight-lang) implementation in Rust. More details about Knight, its license, and specifications can be found in the [knight-lang](https://github.com/knight-lang/knight-lang) repo.

The Rust implementation is a "paragon" implementation: It implements nearly all the suggested extensions—and some compiler-specific quality of life ones—, is able to be embedded within other Rust programs (eg the `discord-bot` directory), and even catches _all_ undefined behaviour.

# Usage
Simply run `cargo run -- (-e 'expr' | -f filename)`, and it'll run your program. Alternatively, you can instead compile the binary with `cargo build`, and then execute it via `./target/debug/knight (-e 'expr' | -f filename)`.

# Enabling strict compliance
By default, the "normal" extensions are enabled, and only simple forms of undefined behaviour are caught. However, you can use `cargo run --no-default-features --features=strict-compliance -- ...` to disable all extensions, and catch _every single form of undefined behaviour_. This can be somewhat slow, however.

# Compiler Flags
The following is the list of flags, and their descriptions. To enable flags, pass `--features=<flag1>,<flag2>,...` as an argument after the `build`/`run`. Note that most of these flags are enabled by default (see the `Cargo.toml`'s `features.default` value for specifics); to disable them, you must run `--no-default-features`, which will only use features you explicitly state

## Spec-defined Extensions
Extensions that the spec suggests.

- `value-function`: Enables the `VALUE` function
- `eval-function`: Enables the `EVAL` function
- `assign-to-strings`: Convert strings to variables if theyre the first arg to `=`.
- `handle-function`: Enables the use of `HANDLE`, which is try-catch.
- `yeet-function`: Enables the use of `YEET`, which is throw.
- `use-function`: Enables the `USE` function, which is for importing files.
- `system-function`: Enables the `$` function, which is for doing system calls.
- **`spec-extensions`**: Enables everything in this section.

## Compiler Extensions
Custom extensions by the Rust implementation that add additional functionality.

- `list-extensions`: Provides useful extensions for lists, such as `* list BLOCK ...` will map all the elements of the list to the return value of the block.
- `string-extensions`: Provides useful extensions for strings, such as `/ string sep` will split a string by the `sep`.
- `xsrand-function`: Enables the use of `XSRAND`, used for seeding `RANDOM`.
- `xrange-function`: Enables the use of `XRANGE`, which is used to construct lists
- `xreverse-function`: Enables the use of `XREVERSE`, which can be used to reverse lists.
- **`compiler-extensions`**: Enables everything in this section.


## Quality-Of-Life Extensions
Extensions that change how Knight itself works, possibly making valid programs invalid.

- `negative-index-length`: Allows the use of negative start positions when indexing into strings/lists; this acts like it does in other languages, such as python.
- `assign-to-prompt`: If you assign to the `PROMPT` function (i.e. `= PROMPT ...`), then the next time `PROMPT` is called, that value will be returned. Multiple assignments start a queue. If a block is assigned, each time `PROMPT` is called, that block is executed, and its return value is the line.
- `assign-to-system`: The same as `assign-to-prompt`, except for the `$` function. If a block is passed, stdin will be assigned to the `_` variable.
- `assign-to-lists`: Allows you to assign to lists of strings, which can be used for argument destructoring.
- `assignment-overloads`: Enables `assign-to-strings`, `assign-to-prompt`, `assign-to-system`, and `assign-to-lists`.
- **`qol-extensions`**: Enables everything in this section.

## Spec Compliance
These extensions can be enabled to make sure knight programs are fully spec compliant.

- `forbid-trailing-tokens`: Raises an error if more than one expression is present in the source code.
- `container-length-limit`: Ensures that containers do not grow larger than `i32::MAX`.
- `strict-charset`: Disables Unicode support, only supporting the strict subset of ASCII Knight requires.
- `checked-overflow`: Checks for under/overflow in integer arithmetic operations.
- `strict-integers`: Use `i32` (instead of the default `i64`) for integers; Also enables some niche integer spec requirements.
- `strict-call-argument`: Ensures that `CALL` is only ever executed with `BLOCK`'s return value.
- `verify-variable-names`: Ensures that all variable names are at most 127 characters long.
- **`strict-compliance`**: Enables everything in this section; With this enabled, and all other features disabled, only 100% valid Knight programs will execute properly.

## Miscellaneous
Miscellaneous extensions that don't fit into other categories

- `multithreaded`: Enable this to turn on multithreading support. Since Knight is normally single threaded, the Rusut implementation by default uses single-threaded data structures such as `Rc` and `RefCell`. 
- `extensions`: Enables `spec-extensions` and `compiler-extensions`, and `qol-extensions`.
