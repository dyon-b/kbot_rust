use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use serenity::constants::GATEWAY_VERSION;
use std::time::Instant;
use crate::helpers::database_helper::DatabaseGuild;
use crate::helpers::global_data::Uptime;
use crate::helpers::general_helper::seconds_to_days;

#[command]
#[description = "Pong!"]
#[aliases("pong", "latency")]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let gateway_url = format!("https://discord.com/api/v{}/gateway", GATEWAY_VERSION);

    // Get latency, Get the gateway URL.
    let now = Instant::now();
    reqwest::get(&gateway_url).await?;
    let get_latency = now.elapsed().as_millis();

    // Post latency, Send a message.
    let now = Instant::now();
    let mut sent_message = msg.channel_id
        .say(&ctx.http, ":hourglass: Calculating latency...").await?;
    let post_latency = now.elapsed().as_millis();

    // Database guild find latency
    let now = Instant::now();
    DatabaseGuild::get(ctx, msg.guild_id.unwrap().0 as i64).await;
    let get_guild_latency = now.elapsed().as_millis();

    sent_message.edit(ctx, |m| {
        m.content("");
        m.embed(|e| {
            e.title("Pong! Latency");
            e.description(format!("REST GET: {}ms\nREST POST: {}ms\nMONGO GET GUILD: {}ms", get_latency, post_latency, get_guild_latency))
        })
    }).await?;

    Ok(())
}

#[command]
#[description = "Some information about the bot."]
#[aliases("info")]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    let avatar_url = ctx.cache.current_user().await.avatar_url();

    let uptime = {
        let instant = {
            let data_read = ctx.data.read().await;
            data_read.get::<Uptime>().unwrap().clone()
        };

        let duration = instant.elapsed();
        seconds_to_days(duration.as_secs())
    };

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("About kBot");
            e.description(format!("**Bot source**\n{}\n**Support server**\n{}",
                                  "https://github.com/kara-b/kBot2", "https://discord.gg/qzGj4En"));
            e.field("Uptime", uptime, true);
            e.thumbnail(avatar_url.unwrap_or_else(String::new));
            e
        });
        m
    }).await?;

    Ok(())
}