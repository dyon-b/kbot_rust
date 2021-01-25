use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{CommandResult, macros::command, Args};
use serenity::utils::Colour;
use crate::helpers::database_helper::{DatabaseGuild, GuildCounting};
use serenity::builder::CreateEmbed;
use crate::helpers::global_data::{CountingCache, PrefixCache};

#[command]
#[description = "Sets the prefix for this server"]
#[required_permissions("ADMINISTRATOR")]
#[max_args(1)]
#[only_in("guilds")]
#[usage = "new_prefix"]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Reset the prefix
    if args.is_empty() {
        // Remove from cache
        ctx.data.read().await.get::<PrefixCache>().unwrap().remove(&msg.guild_id.unwrap());

        // Remove from database
        let mut database_guild = DatabaseGuild::get_or_insert_new(ctx, msg.guild_id.unwrap().0 as i64).await;
        database_guild.prefix = None;

        DatabaseGuild::insert_or_replace(ctx, database_guild).await;

        msg.channel_id.say(ctx, ":white_check_mark: Reset the prefix to the default value.").await?;
    } else {
        let new_prefix = args.single::<String>();
        match new_prefix {
            Err(_) => {
                msg.channel_id.say(ctx, ":no_entry_sign: The prefix provided was not valid.").await?;
            },
            Ok(new_prefix) => {
                // Put it in the cache
                ctx.data.read().await.get::<PrefixCache>().unwrap().insert(msg.guild_id.unwrap(), new_prefix.clone());

                // Put it in the database
                let mut database_guild = DatabaseGuild::get_or_insert_new(ctx, msg.guild_id.unwrap().0 as i64).await;
                database_guild.prefix = Some(new_prefix.clone());

                DatabaseGuild::insert_or_replace(ctx, database_guild).await;

                msg.channel_id.say(ctx, format!(":white_check_mark: Set the prefix to {}", new_prefix)).await?;
            }
        }
    }

    Ok(())
}

#[command]
#[description = "Sets the counting channel"]
#[required_permissions("ADMINISTRATOR")]
#[max_args(1)]
#[only_in("guilds")]
#[aliases("counting")]
#[usage = "channel_id"]
async fn count(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        // Remove from database and cache
        let mut database_guild = DatabaseGuild::get_or_insert_new(ctx, msg.guild_id.unwrap().0 as i64).await;

        // Check if there even is a counting channel
        if database_guild.counting.is_none() {
            msg.channel_id.say(ctx, ":no_entry_sign: There is no counting channel to remove.").await?;
        } else {
            // Remove from cache
            ctx.data.read().await.get::<CountingCache>().unwrap().remove(&ChannelId::from(database_guild.counting.unwrap().channel as u64));

            //Remove from database
            database_guild.counting = None;
            DatabaseGuild::insert_or_replace(ctx, database_guild).await;

            msg.channel_id.say(ctx, ":white_check_mark: Removed the counting channel.").await?;
        }
    } else {
        let new_channel = args.single::<ChannelId>();
        match new_channel {
            Err(why) => {
                msg.channel_id.send_message(&ctx.http, |m| m.embed(|embed| {
                    embed.0 = invalid_channel_embed(why.to_string()).0;
                    embed
                })).await?;
            },
            Ok(new_channel) => {
                let guild = msg.guild(&ctx).await.unwrap();
                if !guild.channels.contains_key(&new_channel) {
                    msg.channel_id.send_message(&ctx.http, |m| m.embed(|embed| {
                        embed.0 = invalid_channel_embed(String::from("Channel not found in this guild.")).0;
                        embed
                    })).await?;
                    return Ok(());
                } else if !guild.user_permissions_in(guild.channels.get(&new_channel).unwrap(), guild.members.get(&ctx.http.get_current_user().await?.id).unwrap()).contains(&Permissions::MANAGE_MESSAGES) {
                    msg.channel_id.send_message(&ctx.http, |m| m.embed(|embed| {
                        embed.0 = invalid_channel_embed(String::from("Missing permissions to delete messages in that channel.")).0;
                        embed
                    })).await?;
                    return Ok(());
                }

                // Insert it into the database
                let mut database_guild = DatabaseGuild::get_or_insert_new(ctx, msg.guild_id.unwrap().0 as i64).await;
                database_guild.counting = Some(GuildCounting { channel: new_channel.0 as i64, count: 0 });

                DatabaseGuild::insert_or_replace(ctx, database_guild).await;

                // Insert into CountingCache
                ctx.data.read().await.get::<CountingCache>().unwrap().insert(new_channel, 0);

                msg.channel_id.say(ctx, format!(":white_check_mark: Set the counting channel to <#{}>", new_channel.0)).await?;
            }
        }
    }

    Ok(())
}
fn invalid_channel_embed(why: String) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    embed.title(":no_entry_sign: Invalid channel.")
        .description(format!("```{}```", why))
        .color(Colour::RED);

    embed
}
