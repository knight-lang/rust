use knightrs::*;

#[derive(Debug)]
struct Foo(i64);

impl<'e> value::custom::CustomType<'e> for Foo {
	fn to_integer(&self, _: &Custom<'e>, _: &mut Environment<'e>) -> Result<Integer> {
		self.0.try_into()
	}

	fn add(&self, _: &Custom<'e>, rhs: &Value<'e>, env: &mut Environment<'e>) -> Result<Value<'e>> {
		Ok(Custom::new(Self(self.0 + i64::from(rhs.to_integer(env)?))).into())
	}
}

fn main() {
	let makeit = Function {
		func: |x, e| Ok(Custom::new(Foo(x[0].run(e)?.to_integer(e)?.into())).into()),
		arity: 1,
		name: "XC".try_into().unwrap(),
	};

	let mut env = Environment::builder();
	env.extensions().insert("C".try_into().unwrap(), &makeit);
	let mut env = env.build();

	let arg = Text::try_from(std::env::args().nth(2).expect("no arg")).unwrap();

	let arg = if std::env::args().nth(1).unwrap() == "-e" {
		arg
	} else {
		std::fs::read_to_string(&**arg).unwrap().as_str().try_into().unwrap()
	};

	match env.play(&arg) {
		Err(Error::Quit(code)) => std::process::exit(code),
		Err(err) => {
			eprintln!("error: {err}");
			std::process::exit(1);
		}
		_ => {}
	}
	drop(env);
	/*
				r##"
	; = å = j 0
	; WHILE < å 100
		; = j + j å
		: = å + å 1
	; O j
	; = a 3
	#: O + a a
	O + 3 "   -4a"

	"##
				.try_into()
				.unwrap(),*/
}
