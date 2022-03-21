use handler::Bot;
use serenity::Client;

mod args;
mod handler;

pub(crate) const BAD_WORDS: &[&str] = &include!("bad-words.json");

#[tokio::main]
async fn main() {
	let args = args::Args::get();
	let mut client = Client::builder(args.bot_token)
		.event_handler(Bot::new())
		.await
		.expect("Failed to init client");

	if let Err(why) = match args.shards {
		Some(n) => client.start_shards(n as u64).await,
		None => client.start_autosharded().await,
	} {
		eprintln!("Failed to start: {}", why)
	}
}
