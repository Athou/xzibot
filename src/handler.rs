use crate::MessageCommand;
use crate::SlashCommand;
use serenity::async_trait;
use serenity::client::Context;
use serenity::client::EventHandler;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::interactions::application_command::ApplicationCommand;
use serenity::model::interactions::Interaction;
use serenity::model::interactions::InteractionResponseType;

pub struct Handler {
    pub slash_commands: Vec<Box<dyn SlashCommand>>,
    pub message_commands: Vec<Box<dyn MessageCommand>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        if message.author.bot {
            return;
        }

        for boxed_message_command in &self.message_commands {
            let message_command = &*boxed_message_command;
            let result = message_command.handle(&ctx, &message);
            match result.await {
                Err(e) => println!("error while executing message command : {}", e),
                Ok(None) => {}
                Ok(Some(r)) => {
                    message.channel_id.say(&ctx.http, r).await.unwrap();
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        ApplicationCommand::set_global_application_commands(&ctx.http, |global_commands| {
            for boxed_slash_command in &self.slash_commands {
                let slash_command = &*boxed_slash_command;
                global_commands.create_application_command(|application_command| {
                    slash_command.register(application_command);
                    application_command
                });
            }
            global_commands
        })
        .await
        .unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(application_command) = interaction {
            println!(
                "command received: {} from {}",
                application_command.data.name, application_command.user.name
            );

            for boxed_slash_command in &self.slash_commands {
                let slash_command = &*boxed_slash_command;
                let result = slash_command.handle(&application_command);
                match result.await {
                    Err(e) => println!(
                        "error while executing command {} : {}",
                        application_command.data.name, e
                    ),
                    Ok(None) => {}
                    Ok(Some(r)) => {
                        application_command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| message.content(r))
                            })
                            .await
                            .unwrap();
                    }
                }
            }
        }
    }
}
