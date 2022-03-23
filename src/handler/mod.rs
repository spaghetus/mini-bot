use std::{collections::HashMap, env, sync::Arc};

use chrono::{DateTime, Duration, Local};

use reqwest::get;
use rust_bert::pipelines::{
	summarization::SummarizationModel,
	text_generation::{TextGenerationConfig, TextGenerationModel},
};
use serenity::{
	async_trait,
	client::EventHandler,
	http::Http,
	model::{
		channel::Message,
		id::{GuildId, UserId},
	},
	prelude::RwLock,
};
use tokio::task;

use crate::BAD_WORDS;

pub struct Bot {
	last_used_by_guild: Arc<RwLock<HashMap<GuildId, DateTime<Local>>>>,
	last_used_by_user: Arc<RwLock<HashMap<UserId, DateTime<Local>>>>,
}

impl Bot {
	pub fn new() -> Bot {
		Bot {
			last_used_by_guild: Arc::new(RwLock::new(HashMap::new())),
			last_used_by_user: Arc::new(RwLock::new(HashMap::new())),
		}
	}
}

#[async_trait]
impl EventHandler for Bot {
	async fn message(
		&self,
		ctx: serenity::client::Context,
		msg: serenity::model::channel::Message,
	) {
		msg.channel_id.start_typing(&ctx.http).unwrap();
		let mentions_me = msg.mentions_me(&ctx.http).await.unwrap();
		let text = msg.content.clone();
		let mut words = text.split(' ').into_iter();
		let at_me = {
			if msg.author.bot {
				false
			} else if mentions_me {
				words.next();
				true
			} else if words.next() == Some("!mini") {
				true
			} else {
				false
			}
		};
		if at_me {
			// Check rate-limiting.
			let (limited_by_user, limited_by_guild) = {
				if msg.author.id.as_u64() == &254656673484898314u64 {
					(false, false)
				} else {
					let now = Local::now();
					let last_used_by_guild = match msg.guild_id {
						Some(id) => {
							let map = self.last_used_by_guild.read().await;
							map.get(&id).cloned()
						}
						None => None,
					};
					let last_used_by_user = self
						.last_used_by_user
						.read()
						.await
						.get(&msg.author.id)
						.cloned();
					let latest_allowed_for_user = now - Duration::seconds(5);
					let latest_allowed_for_guild = now - Duration::seconds(2);
					let limited_by_user = last_used_by_user
						.map(|v| v > latest_allowed_for_user)
						.unwrap_or(false);
					let limited_by_guild = last_used_by_guild
						.map(|v| v > latest_allowed_for_guild)
						.unwrap_or(false);
					(limited_by_user, limited_by_guild)
				}
			};
			match (limited_by_user, limited_by_guild) {
				(false, false) => {}
				(a, b) => {
					safe_say(
						ctx.http,
						&msg,
						match (a, b) {
							(true, true) => "Both you and this server are currently rate-limited.",
							(false, true) => "This server is currently rate-limited.",
							(true, false) => "You are currently rate-limited.",
							(false, false) => "How did we get here? (Reached unreachable code?)",
						},
					)
					.await;
					return;
				}
			}
			// Write rate-limiting information.
			if let Some(id) = msg.guild_id {
				self.last_used_by_guild
					.write()
					.await
					.insert(id, Local::now());
			}
			self.last_used_by_user
				.write()
				.await
				.insert(msg.author.id, Local::now());
			// Process command.
			let command = words.next();
			if command.is_some() || mentions_me {
				match command.unwrap_or("") {
					"hello" => {
						safe_say(ctx.http, &msg, "Hello!").await;
					}
					"echo" => {
						safe_say(ctx.http, &msg, &words.collect::<Vec<&str>>().join(" ")).await;
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
					"minify-web" => {
						let msg = msg.clone();
						let working = safe_say(ctx.http.clone(), &msg, "Working...")
							.await
							.unwrap();
						let next =
							(*words.next().unwrap_or("https://example.com").clone()).to_string();
						let mut content = get(next).await.unwrap().text().await.unwrap();
						if let Some(index) = content.find("</nav>") {
							content = content[index..].to_string();
						}
						let text = html2text::from_read(content.as_bytes(), 200)
							.lines()
							.filter(|v| v.len() > 1)
							.map(|v| v.trim())
							.map(|v| {
								v.chars()
									.filter(|v| {
										v.is_alphabetic()
											|| v.is_ascii_punctuation() || v.is_whitespace()
									})
									.collect::<String>()
							})
							.collect::<Vec<String>>()
							.join(" ");
						if text.len() > 0 {
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
						} else {
							safe_say(ctx.http, &msg, "Empty document?").await;
						}
					}
					"textify" => {
						let msg = msg.clone();
						let working = safe_say(ctx.http.clone(), &msg, "Working...")
							.await
							.unwrap();
						let next =
							(*words.next().unwrap_or("https://example.com").clone()).to_string();
						let mut content = get(next).await.unwrap().text().await.unwrap();
						if let Some(index) = content.find("</nav>") {
							content = content[index..].to_string();
						}
						let text = html2text::from_read(content.as_bytes(), 200)
							.lines()
							.filter(|v| v.len() > 1)
							.map(|v| v.trim())
							.map(|v| {
								v.chars()
									.filter(|v| {
										v.is_alphabetic()
											|| v.is_ascii_punctuation() || v.is_whitespace()
									})
									.collect::<String>()
							})
							.collect::<Vec<String>>()
							.join(" ");
						msg.channel_id
							.delete_message(ctx.http.clone(), working.id)
							.await
							.unwrap();
						if text.len() > 0 {
							safe_say(ctx.http, &msg, &text.chars().take(2000).collect::<String>())
								.await;
						} else {
							safe_say(ctx.http, &msg, "Empty document?").await;
						}
					}
					"complete" => {
						let msg = msg.clone();
						let working = safe_say(ctx.http.clone(), &msg, "Working...")
							.await
							.unwrap();
						let next = words.collect::<Vec<&str>>().join(" ");
						let output = task::spawn_blocking(move || {
							let mut config: TextGenerationConfig = Default::default();
							config.min_length = env::var("MIN_LENGTH")
								.ok()
								.and_then(|v| v.parse::<i64>().ok())
								.unwrap_or(50);
							config.max_length = env::var("MAX_LENGTH")
								.ok()
								.and_then(|v| v.parse::<i64>().ok())
								.unwrap_or(100);
							config.temperature = env::var("TEMPERATURE")
								.ok()
								.and_then(|v| v.parse::<f64>().ok())
								.unwrap_or(4.0);
							let model = TextGenerationModel::new(config).unwrap();
							let inputs = [next.as_str()];
							model.generate(inputs, None).join("\n")
						})
						.await
						.unwrap();
						msg.channel_id
							.delete_message(ctx.http.clone(), working.id)
							.await
							.unwrap();
						safe_say(ctx.http, &msg, &output).await;
					}
					"help" => {
						safe_say(
							ctx.http,
							&msg,
							match words.next() {
								Some("hello") => "Says hello.",
								Some("shard") => "Lets you know which Mini shard you're talking to.",
								Some("minify") => "Tries to compress the text after it. It sometimes doesn't work great...", 
								Some("minify-web") => "Tries to compress the text in the URL after it. It sometimes doesn't work great...", 
								Some("textify") => "Returns the first 2000 characters (Discord length limit) of text in the URL after it.",
								Some("complete") => "Generates more text from the seed after it.",
								Some("help") => "Outputs help text, with an optional topic.",
								Some(_) => "I don't know that help topic, sorry.",
								_ => "Available commands are hello, shard, minify, minify-web, textify, complete, help. You can pass a command's name to help to learn what it does.",
							},
						)
						.await;
					}
					_ => {
						safe_say(
							ctx.http,
							&msg,
							"To learn how to use Mini, type '!mini help'",
						)
						.await;
					}
				}
			}
		}
		()
	}
}

async fn safe_say(http: Arc<Http>, msg: &Message, text: &str) -> Option<Message> {
	let text = text
		.split(' ')
		.map(|w| {
			if BAD_WORDS.contains(&w.to_lowercase().as_str()) {
				"[REDACTED]"
			} else {
				w
			}
		})
		.collect::<Vec<&str>>()
		.join(" ");
	let result = msg.channel_id.say(http, text).await;
	match result {
		Ok(v) => Some(v),
		Err(e) => {
			eprintln!("{}", e);
			None
		}
	}
}
