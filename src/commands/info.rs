use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use std::time::Instant;
use serenity::builder::CreateEmbed;
use serenity::utils::Colour;
use serenity::model::event::EventType::PresencesReplace;

#[command]
#[description = "Gives information about a guild"]
#[only_in("guilds")]
#[aliases("server", "guild", "guildinfo")]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let cached_guild = msg.guild_id.unwrap().to_guild_cached(&ctx.cache).await.unwrap();

    let mut embed = CreateEmbed::default();

    embed.title(&cached_guild.name)
        .thumbnail(&cached_guild.icon_url().unwrap_or(String::new()))
        .color(Colour::BLURPLE)
        .footer(|f| f.text(format!("ID: {}", cached_guild.id.0)));

    // Get the guild owner
    let owner: User = cached_guild.owner_id.to_user(&ctx).await?;
    embed.author(|f| f.name(format!("{}#{} 👑", owner.name, owner.discriminator))
        .icon_url(owner.avatar_url().unwrap_or(String::new())));

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

    embed.field("Members", member_string, true);

    msg.channel_id.send_message(ctx, |m| m.embed(|e| {
        e.0 = embed.0;
        e
    })).await?;

    Ok(())
}