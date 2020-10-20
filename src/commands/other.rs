use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use serenity::constants::GATEWAY_VERSION;
use std::time::Instant;

#[command]
#[description = "Pong!"]
#[aliases("pong")]
async fn ping(ctx: &Context, message: &Message) -> CommandResult {
    let gateway_url = format!("https://discord.com/api/v{}/gateway", GATEWAY_VERSION);

    // Get latency, Get the gateway URL.
    let now = Instant::now();
    reqwest::get(&gateway_url).await?;
    let get_latency = now.elapsed().as_millis();

    // Post latency, Send a message.
    let now = Instant::now();
    let mut sent_message = message.channel_id
        .say(&ctx.http, "Calculating post latency...").await?;
    let post_latency = now.elapsed().as_millis();

    sent_message.edit(ctx, |m| {
        m.content("");
        m.embed(|e| {
            e.title("Pong! Latency");
            e.description(format!("REST GET: {}ms\nREST POST: {}ms", get_latency, post_latency))
        })
    }).await?;

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