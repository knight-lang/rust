use knightrs::env::{Environment, Flags};
use knightrs::value::TextSlice;
use wasm_bindgen::prelude::*;

#[derive(serde::Deserialize)]
struct Options {
	encoding: String,
	inttype: String,
	overflow: String,
	flags: Flags,
}

#[wasm_bindgen]
pub fn play(src: &str, args: JsValue) -> Result<String, String> {
	let Options { encoding, inttype, overflow, flags } =
		serde_wasm_bindgen::from_value(args).map_err(|e| e.to_string())?;
	let mut stdout = Vec::new();

	macro_rules! play {
		(E; "ascii") => (knightrs::value::text::Ascii);
		(E; "knight-encoding") => (knightrs::value::text::KnightEncoding);
		(E; "utf8") => (knightrs::value::text::Utf8);
		(I; "i32") => (i32);
		(I; "i64") => (i64);
		(C; "checked" $x:tt) => (knightrs::value::integer::Checked<play![I; $x]>);
		(C; "wrapping" $x:tt) => (play![I; $x]);
		($($e:tt $i:tt $c:tt),* $(,)?) => {
			match (encoding.as_str(), inttype.as_str(), overflow.as_str()) {
				$(($e, $i, $c) => {
					let mut builder = Environment::<'_, play![C; $c $i], play![E; $e]>::builder(&flags);
					builder.stdout(&mut stdout);
					let mut env = builder.build();
					let result = env.play(TextSlice::new(src, &flags).map_err(|e| e.to_string())?);
					drop(env);
					result.map_err(|e| e.to_string()).map(|_| String::from_utf8_lossy(&stdout).into())
				})*
				_ => Err(format!("bad options: encoding: {encoding:?}, inttype: {inttype:?}, overflow: {overflow:?}"))
			}
		};
	}

	play! {
		"knight-encoding" "i32" "checked",
		"knight-encoding" "i32" "wrapping",
		"knight-encoding" "i64" "checked",
		"knight-encoding" "i64" "wrapping",

		"ascii" "i32" "checked",
		"ascii" "i32" "wrapping",
		"ascii" "i64" "checked",
		"ascii" "i64" "wrapping",

		"utf8" "i32" "checked",
		"utf8" "i32" "wrapping",
		"utf8" "i64" "checked",
		"utf8" "i64" "wrapping",
	}
}
