fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=bad-words.url");
	let url = std::fs::read_to_string("bad-words.url").unwrap();
	let bad_words_json = reqwest::blocking::get(url).unwrap().text().unwrap();
	std::fs::write("src/bad-words.json", bad_words_json).unwrap();
}