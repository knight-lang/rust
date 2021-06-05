use super::{Parser, ParseResult, Character};
use crate::Value;

pub type TokenizeFn = for<'env> fn(&mut Parser<'env, '_>, Character) -> ParseResult<Value<'env>>;

pub fn whitespace<'env>(parser: &mut Parser<'env, '_>, _: Character) -> ParseResult<Value<'env>> {
	parser.next_value()
}

pub fn comment<'env>(parser: &mut Parser<'env, '_>, _: Character) -> ParseResult<Value<'env>> {
	const EOL: Character = b'\n' as _;

	while !matches!(parser.next_character(), Some(EOL) | None) {
		/* do nothing */
	}

	parser.next_value()
}

pub fn number<'env>(parser: &mut Parser<'env, '_>, first: Character) -> ParseResult<Value<'env>> {
	let _ = (parser, first);

	todo!()
}

pub fn text<'env>(parser: &mut Parser<'env, '_>, quote: Character) -> ParseResult<Value<'env>> {
	let _ = (parser, quote);

	todo!()
}

pub fn variable<'env>(parser: &mut Parser<'env, '_>, first: Character) -> ParseResult<Value<'env>> {
	let _ = (parser, first);

	todo!()
}

pub fn function<'env>(parser: &mut Parser<'env, '_>, which: Character) -> ParseResult<Value<'env>> {
	let _ = (parser, which);

	todo!()
}