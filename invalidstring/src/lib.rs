/// uses unsafe to postfix the string with invalid utf8 data
pub fn invalid_utf8(prefix: &str) -> String {
	let bytes = b"\xc3\x73";

	unsafe {
		format!("{}{}", prefix, std::str::from_utf8_unchecked(bytes))
	}
}
