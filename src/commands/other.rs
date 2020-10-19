use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};

#[command]
#[description = "Pong!"]
#[aliases("pong")]
async fn ping(ctx: &Context, message: &Message) -> CommandResult {
    message.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

#[command]
#[description = "Some information about the bot."]
#[aliases("info")]
async fn about(ctx: &Context, message: &Message) -> CommandResult {
    let avatar_url = ctx.cache.current_user().await.avatar_url();

    message.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("About kBot");
            e.description(format!("**Bot source**\n{}\n**Support server**\n{}",
                                  "https://github.com/kara-b/kBot2", "https://discord.gg/qzGj4En"));
            e.thumbnail(avatar_url.unwrap_or_else(String::new));
            e
        });
        m
    }).await?;

    Ok(())
}