use std::env;

use links::find_platform_links;
use serenity::all::{CommandInteraction, CommandType, CreateCommand};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{Command, Interaction};
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod links;

const COMMAND_NAME: &str = "Alt URLs";

async fn handle_alt_urls_command(ctx: Context, interaction: CommandInteraction) {
    let messages = interaction.data.resolved.messages.values();
    let links = messages
        .flat_map(|msg| find_platform_links(&msg.content))
        .flat_map(|link| link.alternative_links())
        .map(|link| link.to_string())
        .collect::<Vec<_>>();

    let data = if links.is_empty() {
        CreateInteractionResponseMessage::new()
            .content("No supported links found in message :(")
            .ephemeral(true)
    } else {
        CreateInteractionResponseMessage::new().content(links.join("\n"))
    };
    let builder = CreateInteractionResponse::Message(data);
    if let Err(why) = interaction.create_response(&ctx.http, builder).await {
        println!("Cannot respond to slash command: {why}");
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if let (COMMAND_NAME, CommandType::Message) =
                (command.data.name.as_str(), command.data.kind)
            {
                handle_alt_urls_command(ctx, command).await
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // register message command
        let alt_url_command = CreateCommand::new(COMMAND_NAME).kind(CommandType::Message);
        Command::create_global_command(&ctx.http, alt_url_command)
            .await
            .expect("Failed to register message command");
    }
}

#[tokio::main]
async fn main() {
    let token =
        env::var("DISCORD_TOKEN").expect("Please set the environment variable DISCORD_TOKEN");

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
