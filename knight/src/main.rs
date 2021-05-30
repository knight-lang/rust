use knight::*;

fn main() {
    println!("{:?}", Value::from(unsafe { Text::new_unchecked(std::borrow::Cow::Borrowed("foo")) }));
}
