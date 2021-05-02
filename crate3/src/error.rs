#[derive(Debug)]
pub enum Error {
	UndefinedVariable(String)
}

pub type Result<T> = std::result::Result<T, Error>;
