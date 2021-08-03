use clap::{load_yaml, App};

pub struct Args {
	pub bot_token: String,
	pub shards: Option<usize>,
}
impl Args {
	pub fn get() -> Args {
		let yaml = load_yaml!("cli.yaml");
		let matches = App::from_yaml(yaml).get_matches();
		Args {
			bot_token: matches.value_of("token").unwrap().to_string(),
			shards: matches.value_of("shards").map(|v| v.parse().unwrap()),
		}
	}
}
