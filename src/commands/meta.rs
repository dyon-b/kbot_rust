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
use serenity::builder::CreateEmbed;
use serenity::utils::Colour;
use serenity::model::Permissions;

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
        .say(&ctx.http, "<a:loading:776804948633059338> Calculating latency...").await?;
    let post_latency = now.elapsed().as_millis();

    // Database guild find latency, Absolutely cursed.
    let mut guild_string = String::from("");
    if msg.guild_id != None {
        let now = Instant::now();
        DatabaseGuild::get(ctx, msg.guild_id.unwrap().0 as i64).await;
        let get_guild_latency = now.elapsed().as_millis();
        guild_string = format!("\nMONGO GET GUILD: {}ms", get_guild_latency);
    }

    sent_message.edit(&ctx, |m| m.content("").embed(|e| {
        e.title("Pong! Latency");
        e.description(format!("REST GET: {}ms\nREST POST: {}ms{}", get_latency, post_latency, guild_string));
        e.color(Colour::BLURPLE)
    })).await?;

    Ok(())
}

#[command]
#[description = "Some information about the bot."]
#[aliases("info", "stats", "uptime", "botinfo")]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    let mut message = msg.channel_id.say(&ctx.http, "<a:loading:776804948633059338> Collecting data...").await?;
    let mut embed = CreateEmbed::default();

    // Basic info
    embed.title("kBot");
    embed.url("https://github.com/kara-b/kbot_rust");
    embed.footer(|f| f.text("A general purpose bot made with Rust, Serenity and love.")
        .icon_url("https://raw.githubusercontent.com/serenity-rs/serenity/current/logo.png"));
    embed.color(Colour::BLURPLE);

    // Owner info
    let owner = &ctx.http.get_current_application_info().await?.owner;
    let owner_name = format!("{}#{}", owner.name, owner.discriminator);
    let owner_avatar = owner.avatar_url().unwrap_or(String::new());
    embed.author(|f| f.name(owner_name).icon_url(owner_avatar));

    // Channels and member count
    let mut text_channels: i32 = 0;
    let mut voice_channels: i32 = 0;
    for guild_id in &ctx.cache.guilds().await {
        let cached_guild = guild_id.to_guild_cached(&ctx.cache).await;
        if cached_guild.is_some() {
            let channels  = cached_guild.unwrap().channels;
            for channel in channels {
                let channel = channel.1;
                if channel.kind == ChannelType::Text {
                    text_channels += 1;
                } else if channel.kind == ChannelType::Voice {
                    voice_channels += 1;
                }
            }
        }
    }
    embed.field("Cached channels", format!("{} total\n{} text\n{} voice", (text_channels + voice_channels), text_channels, voice_channels), true);

    // Member count
    let unknown_members = ctx.cache.unknown_members().await as usize;
    let cached_members = ctx.cache.user_count().await;
    embed.field("Members", format!("{} unknown\n{} cached", unknown_members, cached_members), true);

    // Guilds count
    let guilds_count = &ctx.cache.guilds().await.len();
    embed.field("Guilds", guilds_count, true);

    // Uptime
    let uptime = {
        let instant = {
            let data_read = ctx.data.read().await;
            data_read.get::<Uptime>().unwrap().clone()
        };

        let duration = instant.elapsed();
        seconds_to_days(duration.as_secs())
    };
    embed.field("Uptime", uptime, true);

    message.edit(&ctx, |m| m.content("").embed(|e| {
        e.0 = embed.0;
        e
    })).await?;

    Ok(())
}

#[command]
#[description = "Gives information about a guild"]
#[only_in("guilds")]
#[aliases("server", "guild", "guildinfo")]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let mut message = msg.channel_id.say(&ctx.http, "<a:loading:776804948633059338> Loading information about the guild...").await?;

    let cached_guild = msg.guild_id.unwrap().to_guild_cached(&ctx.cache).await.unwrap();

    let mut embed = CreateEmbed::default();

    embed.title(&cached_guild.name)
        .thumbnail(&cached_guild.icon_url().unwrap_or(String::new()))
        .color(Colour::BLURPLE)
        .footer(|f| f.text(format!("ID: {} Created", cached_guild.id.0)))
        .timestamp(&msg.guild_id.unwrap().created_at());

    // Get the guild owner
    let owner: User = cached_guild.owner_id.to_user(&ctx).await?;
    embed.author(|f| f.name(format!("{}#{} 👑", owner.name, owner.discriminator))
        .icon_url(owner.avatar_url().unwrap_or(String::new())));

    // Emote list
    let mut animated_emotes = 0;
    let mut regular_emotes = 0;
    for emoji in cached_guild.emojis {
        if emoji.1.animated { animated_emotes += 1; } else { regular_emotes += 1; };
    }
    let emoji_limit = cached_guild.premium_tier.num() * 50 + 50;
    let emote_string = format!("Regular: {}/{}\nAnimated: {}/{}", regular_emotes, emoji_limit, animated_emotes, emoji_limit);
    embed.field("Emotes", emote_string, true);

    // Collect the channel count from cache to be speedy
    let mut text_channels = 0;
    let mut voice_channels = 0;

    for channel in &cached_guild.channels {
        let channel = channel.1;
        if channel.kind == ChannelType::Text {
            text_channels += 1;
        } else if channel.kind == ChannelType::Voice {
            voice_channels += 1;
        }
    }
    let channels_text = format!("<:text_channel:776807879146471434> {}\n\
    <:voice_channel:776808150631448576> {}", text_channels, voice_channels);
    embed.field("Channels", channels_text, true);

    // Collect the member count
    let mut bot_count = 0;
    let mut human_count = 0;

    let mut online_count = 0;
    let mut idle_count = 0;
    let mut dnd_count = 0;
    let mut offline_count = 0;

    for member_result in &cached_guild.members {
        if member_result.1.user.bot { bot_count += 1 } else { human_count += 1 };

        match cached_guild.presences.get(member_result.0) {
            Some(presence) => {
                match presence.status {
                    OnlineStatus::Online => online_count += 1,
                    OnlineStatus::DoNotDisturb => dnd_count += 1,
                    OnlineStatus::Idle => idle_count += 1,
                    OnlineStatus::Offline => offline_count += 1,
                    OnlineStatus::Invisible => offline_count += 1,
                    _ => {}
                }
            }
            None => { offline_count += 1; }
        }
    }
    let member_count = bot_count + human_count;
    // Add the member count to the embed
    let member_string = format!("<:status_online:776752278681813032> {} \
    <:status_idle:776752682244636715> {} \
    <:status_dnd:776752681808560138> {} \
    <:status_offline:776752682584899604> {}\n\
    {} humans {} bots {} total", online_count, idle_count, dnd_count, offline_count, human_count, bot_count, member_count);

    embed.field("Members", member_string, false);

    // Boosts
    let boosts_string = format!("Level {}\n{} boosts", cached_guild.premium_tier.num(), cached_guild.premium_subscription_count);
    embed.field("Boosts", boosts_string, true);

    // Role count
    embed.field("Roles", format!("{} roles", cached_guild.roles.len()), true);

    // Send the embed
    message.edit(&ctx, |f| f.content("").embed(|e| {
        e.0 = embed.0;
        e
    })).await?;

    Ok(())
}

#[command]
#[description = "Gives you an invite link"]
async fn invite(ctx: &Context, msg: &Message) -> CommandResult {
    let mut permissions = Permissions::default();
    permissions.set(Permissions::READ_MESSAGES, true);
    permissions.set(Permissions::SEND_MESSAGES, true);
    permissions.set(Permissions::MANAGE_MESSAGES, true);
    permissions.set(Permissions::EMBED_LINKS, true);
    permissions.set(Permissions::READ_MESSAGE_HISTORY, true);
    permissions.set(Permissions::ADD_REACTIONS, true);
    permissions.set(Permissions::USE_EXTERNAL_EMOJIS, true);

    let invite_url = match ctx.cache.current_user().await.invite_url(ctx, permissions).await {
        Ok(v) => v,
        Err(why) => {
            println!("Error creating invite url: {:?}", why);

            msg.channel_id.say(&ctx, ":no_entry_sign: Error creating invite url").await?;

            return Ok(());
        }
    };

    msg.channel_id.send_message(&ctx, |m| m.embed(|e| {
        e.title("Invite link")
            .url(invite_url)
            .color(Colour::BLURPLE)
    })).await?;

    Ok(())
}