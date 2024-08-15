use serenity::{
    all::{
        Command, CommandInteraction, CommandType, Context, CreateCommand,
        CreateInteractionResponse, CreateInteractionResponseMessage, Ready,
    },
    async_trait,
    futures::future,
};

use crate::links::find_platform_links;

use super::RRCommandInteraction;

const MSG_COMMAND_NAME: &str = "Alt URLs";
pub struct MsgCommand;

#[async_trait]
impl RRCommandInteraction for MsgCommand {
    fn name(&self) -> String {
        "message command".to_owned()
    }

    fn can_handle(&self, interaction: &CommandInteraction) -> bool {
        interaction.data.name.as_str() == MSG_COMMAND_NAME
            && interaction.data.kind == CommandType::Message
    }

    async fn handle_impl(
        &self,
        interaction: &CommandInteraction,
    ) -> Result<CreateInteractionResponse, String> {
        debug_assert!(self.can_handle(interaction));

        let messages = interaction.data.resolved.messages.values();
        let alt_urls = future::join_all(
            messages
                .flat_map(|msg| find_platform_links(&msg.content))
                .map(|link| link.alternative_links()),
        )
        .await
        .into_iter()
        .flatten()
        .map(|link| link.to_string())
        .collect::<Vec<_>>();

        if alt_urls.is_empty() {
            Err("Provided links are not supported :(".to_owned())
        } else {
            let reply_msg = CreateInteractionResponseMessage::new().content(alt_urls.join("\n"));
            Ok(CreateInteractionResponse::Message(reply_msg))
        }
    }

    async fn register(&self, ctx: &Context, _ready: &Ready) -> Result<(), serenity::Error> {
        let msg_command = CreateCommand::new(MSG_COMMAND_NAME).kind(CommandType::Message);
        Command::create_global_command(&ctx.http, msg_command).await?;

        Ok(())
    }
}
