use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use std::time::Instant;

#[command]
#[description = "Gives information about a guild"]
#[only_in("guilds")]
#[aliases("server", "guild", "guildinfo")]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let cached_guild = msg.guild_id.unwrap().to_guild_cached(&ctx.cache).await.unwrap();

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

    for member_result in &cached_guild.members {
        if member_result.1.user.bot { bot_count += 1 } else { human_count += 1 };
    }
    let member_count = bot_count + human_count;

    // Get the guild owner
    let owner: User = cached_guild.owner_id.to_user(&ctx).await?;

    msg.channel_id.send_message(ctx, |m| m.embed(|e| {
        e.title(format!("Server info for {}", cached_guild.name))
            .thumbnail(cached_guild.icon_url().unwrap())
            .field("Owner", format!("{}#{}", owner.name, owner.discriminator), true)
            .field("Region", &cached_guild.region, true)
            .field("Channels", format!("**Text:** {}\n**Voice:** {}\n", text_channels, voice_channels), true)
            .field("Members", format!("**Total:** {}\n**Humans:** {}\n**Bots:** {}\n", member_count, human_count, bot_count), true)
            .footer(|f| f.text(format!("ID: {}", cached_guild.id.0)))
    })).await?;

    Ok(())
}