use std::env;

use serenity::async_trait;
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod links;

mod interactions;
use interactions::{MsgCommand, RRCommandInteraction};

#[tokio::main]
async fn main() {
    let token =
        env::var("DISCORD_TOKEN").expect("Please set the environment variable DISCORD_TOKEN");

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler::new())
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

struct Handler {
    command_interactions: Vec<Box<dyn RRCommandInteraction + Sync + Send>>,
}

impl Handler {
    fn new() -> Self {
        Self {
            command_interactions: vec![Box::new(MsgCommand)],
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            for interaction in &self.command_interactions {
                if interaction.can_handle(&command) {
                    if let Err(e) = interaction.handle(&ctx, &command).await {
                        println!("failed to handle {}: {e}", interaction.name());
                    }
                    break;
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        for interaction in &self.command_interactions {
            if let Err(e) = interaction.register(&ctx, &ready).await {
                println!("Failed to register {}: {e}", interaction.name());
            }
        }
    }
}
