use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use serenity::constants::GATEWAY_VERSION;
use std::time::Instant;
use std::fs;
use toml::Value as TomlValue;
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

    // Database guild find latency, Absolutely cursed.
    let mut guild_string = String::from("");
    if msg.guild_id != None {
        let now = Instant::now();
        DatabaseGuild::get(ctx, msg.guild_id.unwrap().0 as i64).await;
        let get_guild_latency = now.elapsed().as_millis();
        guild_string = format!("\nMONGO GET GUILD: {}ms", get_guild_latency);
    }

    sent_message.edit(ctx, |m| {
        m.content("");
        m.embed(|e| {
            e.title("Pong! Latency");
            e.description(format!("REST GET: {}ms\nREST POST: {}ms{}", get_latency, post_latency, guild_string))
        })
    }).await?;

    Ok(())
}

#[command]
#[description = "Some information about the bot."]
#[aliases("info", "stats")]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    let avatar_url = ctx.cache.current_user().await.avatar_url();

    // Uptime
    let uptime = {
        let instant = {
            let data_read = ctx.data.read().await;
            data_read.get::<Uptime>().unwrap().clone()
        };

        let duration = instant.elapsed();
        seconds_to_days(duration.as_secs())
    };

    // Guild count, Channel count and user count.
    let guilds_count = &ctx.cache.guilds().await.len();
    let channels_count = &ctx.cache.guild_channel_count().await;
    let users_count = ctx.cache.user_count().await;
    let users_count_unknown = ctx.cache.unknown_members().await as usize;

    // Read Cargo.toml
    let cargo_toml: TomlValue = toml::from_slice(&fs::read("Cargo.toml")?)?;
    let serenity_version = cargo_toml["dependencies"]["serenity"]["version"].as_str().unwrap();

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("About kBot");
            e.description(format!("A general purpose bot made with [Rust]({}), [Serenity v{}]({}) and love.\n\
            You can find the bot's source [here]({}).",
                                  "https://www.rust-lang.org/", serenity_version, "https://github.com/serenity-rs/serenity", "https://github.com/kara-b/kbot_rust"));
            e.field("Statistics", format!("Guilds: {}\nChannels: {}\nTotal Users: {}\nCached Users: {}",
            guilds_count, channels_count, users_count + users_count_unknown, users_count), true);
            e.field("Uptime", uptime, true);
            e.thumbnail(avatar_url.unwrap_or_else(String::new));
            e
        });
        m
    }).await?;

    Ok(())
}