#![allow(unused)]
use super::*;
use std::thread::{self, ScopedJoinHandle};

pub struct Thread<'q, I>(ScopedJoinHandle<'q, Value<I>>);

impl<'q, I: IntType, E: Encoding> Thread<'q, I> {
	#[cfg(any())]
	pub fn spawn(body: Value<I>, env: &'q mut crate::env::Environment<I>) -> Self {
		// Self(thread::spawn(move || body.run(env).unwrap()))
		Self(thread::scope::<'e>(move |s| s.spawn(move || body.run(env).unwrap())))
	}
}

// #[cfg(feature = "extensions")]
// #[cfg_attr(doc_cfg, doc(cfg(feature = "extensions")))]
// pub fn XRANGE< I: IntType, E: Encoding>() -> ExtensionFunction< I> {
// 	xfunction!("XRANGE", env, |start, stop| {
// 		match start.run(env)? {
// 			Value::Integer(start) => {
// 				let stop = stop.run(env)?.to_integer(env)?;

// 				match start <= stop {
// 					true => List::new(
// 						(i64::from(start)..i64::from(stop))
// 							.map(|x| Value::from(crate::value::Integer::try_from(x).unwrap()))
// 							.collect::<Vec<Value< I>>>(),
// 						env.flags(),
// 					)
// 					.expect("todo: out of bounds error")
// 					.into(),

// 					false => {
// 						// (stop..start).map(Value::from).rev().collect::<List>().into()
// 						todo!()
// 					}
// 				}
// 			}

// 			Value::Text(_text) => {
// 				// let start = text.get(0).a;
// 				todo!()
// 			}

// 			other => return Err(Error::TypeError(other.typename(), "XRANGE")),
// 		}
// 	})
// }
