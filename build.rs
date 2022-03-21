fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=bad-words.url");
	let url = std::fs::read_to_string("bad-words.url").unwrap();
	let bad_words_csv = reqwest::blocking::get(url)
		.unwrap()
		.text()
		.unwrap()
		.lines()
		.skip(1)
		.flat_map(|l| l.split(',').next().map(|v| v.to_string()))
		.chain(
			std::fs::read_to_string("more-bad-words.list")
				.unwrap()
				.lines()
				.map(|v| v.to_string()),
		)
		.collect::<Vec<String>>();
	let bad_words_json = serde_json::to_string(&bad_words_csv).unwrap();
	std::fs::write("src/bad-words.json", bad_words_json).unwrap();
}
