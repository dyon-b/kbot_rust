use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use serenity::futures::{StreamExt};
use serenity::FutureExt;

#[command]
#[description = "Gives information about a guild"]
#[only_in("guilds")]
#[aliases("server", "guild", "guildinfo")]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let mut text_channels = 0;
    let mut voice_channels = 0;

    // Collect the channel count
    let mut channels = msg.guild_id.unwrap().channels(ctx).await?;
    for channel in channels {
        let channel = channel.1;
        if channel.kind == ChannelType::Text {
            text_channels += 1;
        } else if channel.kind == ChannelType::Voice {
            voice_channels += 1;
        }
    }

    let mut bot_count = 0;
    let mut human_count = 0;

    // Collect the member count
    let mut members_stream = msg.guild_id.unwrap().members_iter(&ctx).boxed();
    while let Some(member_result) = members_stream.next().await {
        match member_result {
            Ok(member) => {
                if member.user.bot { bot_count += 1 } else { human_count += 1 }
            },
            Err(why) => eprintln!("An error occurred when iterating through members: {}", why),
        }
    }
    let member_count = bot_count + human_count;

    let guild = msg.guild(ctx).await.unwrap();
    let owner: User = guild.owner_id.to_user(ctx).await?;

    msg.channel_id.send_message(ctx, |m| m.embed(|e| {
        e.title(format!("Server info for {}", guild.name))
            .thumbnail(guild.icon_url().unwrap())
            .field("Owner", format!("{}#{}", owner.name, owner.discriminator), true)
            .field("Region", &guild.region, true)
            .field("Channels", format!("**Text:** {}\n**Voice:** {}\n", text_channels, voice_channels), true)
            .field("Members", format!("**Total:** {}\n**Humans:** {}\n**Bots:** {}\n", member_count, human_count, bot_count), true)
            .footer(|f| f.text(format!("ID: {}", guild.id.0)))
    })).await?;

    Ok(())
}