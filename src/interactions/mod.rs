use serenity::{
    all::{
        CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage, Ready,
    },
    async_trait,
};

mod msg_command;
pub(crate) use msg_command::MsgCommand;

#[async_trait]
pub trait RRCommandInteraction {
    fn name(&self) -> String;

    fn can_handle(&self, interaction: &CommandInteraction) -> bool;

    async fn handle_impl(
        &self,
        interaction: &CommandInteraction,
    ) -> Result<CreateInteractionResponse, String>;
    async fn handle(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<(), serenity::Error> {
        let response = match self.handle_impl(interaction).await {
            Ok(r) => r,
            Err(e) => CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("Error: {e}"))
                    .ephemeral(true),
            ),
        };
        interaction.create_response(&ctx.http, response).await?;
        Ok(())
    }

    async fn register(&self, ctx: &Context, ready: &Ready) -> Result<(), serenity::Error>;
}
