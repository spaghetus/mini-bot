use std::sync::Arc;

use rust_bert::pipelines::summarization::SummarizationModel;
use serenity::{async_trait, client::EventHandler, http::Http, model::channel::Message};
use tokio::task;

pub struct Bot;

#[async_trait]
impl EventHandler for Bot {
	async fn message(
		&self,
		ctx: serenity::client::Context,
		msg: serenity::model::channel::Message,
	) {
		msg.channel_id.start_typing(&ctx.http).unwrap();
		let mut words = msg.content.split(' ');
		if words.next() == Some("!mini") {
			if let Some(command) = words.next() {
				match command {
					"hello" => {
						safe_say(ctx.http, &msg, "Hello!").await;
					}
					"shard" => {
						safe_say(ctx.http, &msg, &ctx.shard_id.to_string()).await;
					}
					"minify" => {
						let msg = msg.clone();
						let working = safe_say(ctx.http.clone(), &msg, "Working...")
							.await
							.unwrap();
						let text = words.collect::<Vec<&str>>().join(" ");
						let output = task::spawn_blocking(move || {
							let model = SummarizationModel::new(Default::default()).unwrap();
							let inputs = [text.as_str()];
							model.summarize(inputs)[0].clone()
						})
						.await
						.unwrap();
						msg.channel_id
							.delete_message(ctx.http.clone(), working.id)
							.await
							.unwrap();
						safe_say(ctx.http, &msg, &output).await;
					}
					_ => {
						safe_say(ctx.http, &msg, "I don't know that command, sorry.").await;
					}
				}
			} else {
				safe_say(ctx.http, &msg, "Please specify a command.").await;
			}
		}
		()
	}
}

async fn safe_say(http: Arc<Http>, msg: &Message, text: &str) -> Option<Message> {
	let result = msg.channel_id.say(http, text).await;
	match result {
		Ok(v) => Some(v),
		Err(e) => {
			eprintln!("{}", e);
			None
		}
	}
}
