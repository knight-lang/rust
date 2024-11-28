/// A trait that indicates that something can be parsed.
pub trait Parsable: Sized {
	/// The type that's being parsed.
	type Output;

	/// Attempt to parse an `Output` from the `parser`.
	///
	/// - If an `Output` was successfully parsed, then return `Ok(Some(...))`.
	/// - If there's nothing applicable to parse from `parser`, then `Ok(None)` should be returned.
	/// - If parsing should be restarted from the top (e.g. the [`Blank`] parser removing
	///   whitespace), then [`ErrorKind::RestartParsing`] should be returned.
	/// - If there's an issue when parsing (such as missing a closing quote), an [`Error`] should be
	///   returned.
	fn parse(parser: &mut Parser<'_, '_>) -> Result<Option<Self::Output>>;

	/// A convenience function that generates things you can stick into [`env::Builder::parsers`](
	/// crate::env::Builder::parsers).
	fn parse_fn() -> ParseFn
	where
		Value: From<Self::Output>,
	{
		RefCount::new(|parser| Ok(Self::parse(parser)?.map(Value::from)))
	}
}
