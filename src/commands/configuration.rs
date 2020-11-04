use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{CommandResult, macros::command, Args};
use crate::helpers::database_helper::DatabaseGuild;

#[command]
#[description = "Sets the prefix for this server"]
#[required_permissions("ADMINISTRATOR")]
#[max_args(1)]
#[only_in("guilds")]
#[usage = "new_prefix"]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
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
                let mut database_guild = DatabaseGuild::get_or_insert_new(ctx, msg.guild_id.unwrap().0 as i64).await;
                database_guild.prefix = Some(new_prefix.clone());

                DatabaseGuild::insert_or_replace(ctx, database_guild).await;

                msg.channel_id.say(ctx, format!(":white_check_mark: Set the prefix to {}", new_prefix)).await?;
            }
        }
    }

    Ok(())
}