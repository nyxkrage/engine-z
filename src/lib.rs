pub fn hello_world() -> &'static str { "Hello World!" }

#[cfg(test)]
pub mod tests {
	#[test]
	fn test_hello_world() {
		assert_eq!(crate::hello_world(), "Hello World!");
	}

	#[test]
	fn test_hello_john() {
		assert_ne!(crate::hello_world(), "Hello John!");
	}
}
