use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use serenity::builder::CreateEmbed;
use serenity::utils::Colour;

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