use knightrs::Environment;
use std::time::Duration;

use serenity::{
	 async_trait,
	 model::{channel::Message, gateway::Ready},
	 prelude::*,
};

struct Handler;

fn run_str(cmd: &str) -> String {
	let mut stdout = Vec::new();
	let mut stdin = std::io::Cursor::new("");

	Environment::builder()
		.stdin(&mut stdin)
		.stdout(&mut stdout)
		.disable_system()
		.build()
		.run_str(cmd)
		.map(|_| format!("result:\n```\n{}\n```", String::from_utf8_lossy(&stdout)))
		.unwrap_or_else(|err| format!("error: {}", err))
}

#[async_trait]
impl EventHandler for Handler {
	 async fn message(&self, ctx: Context, msg: Message) {
		if !msg.mentions.iter().any(|user| user.id == 830924836137467904u64) {
			return;
		}

		let cmd =
			if let Some(cmd) =
				msg.content
					.splitn(3, "```")
					.skip(1) // ignore prefix
					.next() // next is the contents
					.or_else(|| msg.content.splitn(3, "`").skip(1).next())
			{
				cmd.to_owned()
			} else {
				return;
			};

    let response_fut = tokio::spawn(async move { run_str(&cmd) });
    let response = tokio::time::timeout(Duration::from_secs(5), response_fut).await
    		.map(|x| x.unwrap())
 			.unwrap_or_else(|_| "error: timed out".to_string());

		if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
			println!("Error sending message: {:?}", why);
		}
	}

	 async fn ready(&self, _: Context, ready: Ready) {
		  println!("{} is connected!", ready.user.name);
	 }
}

#[tokio::main]
async fn main() {
	 // Configure the client with your Discord bot token in the environment.
	 let token = std::env::var("DISCORD_TOKEN")
			.unwrap_or(include_str!("../token.ignore").to_owned());
		  // .expect("Expected a token in the environment");

	 // Create a new instance of the Client, logging in as a bot. This will
	 // automatically prepend your bot token with "Bot ", which is a requirement
	 // by Discord for bot users.
	 let mut client = Client::builder(&token)
		  .event_handler(Handler)
		  .await
		  .expect("Err creating client");


	 // Finally, start a single shard, and start listening to events.
	 //
	 // Shards will automatically attempt to reconnect, and will perform
	 // exponential backoff until it reconnects.
	 if let Err(why) = client.start().await {
		  println!("Client error: {:?}", why);
	 }
}
