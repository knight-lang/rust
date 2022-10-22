use knightrs::env::{Environment, Flags};
use knightrs::value::TextSlice;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn play(src: &str, args: JsValue) -> Result<String, String> {
	let flags: Flags = serde_wasm_bindgen::from_value(args).map_err(|e| e.to_string())?;
	let mut stdout = Vec::new();
	let mut builder = Environment::builder(&flags);
	builder.stdout(&mut stdout);
	let mut env = builder.build();
	let result = env.play(TextSlice::new(src, &flags).map_err(|e| e.to_string())?);
	drop(env);
	result.map_err(|e| e.to_string()).map(|_| String::from_utf8_lossy(&stdout).into())
}
