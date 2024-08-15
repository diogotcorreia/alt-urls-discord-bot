use serenity::{
    all::{
        Command, CommandInteraction, CommandOptionType, CommandType, Context, CreateCommand,
        CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage, Ready,
    },
    async_trait,
};
use url::Url;

use crate::links::PlatformLink;

use super::RRCommandInteraction;

const SLASH_COMMAND_NAME: &str = "alturls";
pub struct SlashCommand;

#[async_trait]
impl RRCommandInteraction for SlashCommand {
    fn name(&self) -> String {
        "slash command".to_owned()
    }

    fn can_handle(&self, interaction: &CommandInteraction) -> bool {
        if interaction.data.name.as_str() != SLASH_COMMAND_NAME
            || interaction.data.kind != CommandType::ChatInput
        {
            return false;
        }

        // sanity checks
        if interaction.data.options.len() != 1 {
            println!("slash command should have 1 option");
            return false;
        }
        if interaction.data.options[0].kind() != CommandOptionType::String {
            println!("slash command should have 1 option");
            return false;
        }

        true
    }

    async fn handle_impl(
        &self,
        interaction: &CommandInteraction,
    ) -> Result<CreateInteractionResponse, String> {
        debug_assert!(self.can_handle(interaction));

        let url = interaction.data.options[0].value.as_str().unwrap();
        let url = Url::parse(url).map_err(|e| format!("failed to parse url: {e}"))?;
        let link = PlatformLink::try_from(url)
            .map_err(|e| format!("failed to parse plaform link: {e}"))?;
        let alt_urls = link.alternative_links().await;

        if alt_urls.is_empty() {
            Err("Provided link is not supported :(".to_owned())
        } else {
            let content = alt_urls
                .into_iter()
                .map(|link| link.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            let reply_msg = CreateInteractionResponseMessage::new().content(content);
            Ok(CreateInteractionResponse::Message(reply_msg))
        }
    }

    async fn register(&self, ctx: &Context, _ready: &Ready) -> Result<(), serenity::Error> {
        let slash_command = CreateCommand::new(SLASH_COMMAND_NAME)
            .kind(CommandType::ChatInput)
            .description("Get alternative URLs for the provided link")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "url",
                    "The URL to get alternative links for",
                )
                .required(true),
            );
        Command::create_global_command(&ctx.http, slash_command).await?;

        Ok(())
    }
}
