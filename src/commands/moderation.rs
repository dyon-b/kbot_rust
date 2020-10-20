use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{CommandResult, macros::command, Args};

#[command]
#[required_permissions("MANAGE_MESSAGES")]
#[num_args(1)]
#[only_in("guilds")]
#[aliases("prune", "clear")]
#[usage = "amount"]
async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let delete_num = args.single::<u64>();
    match delete_num {
        Err(_) => { msg.channel_id.say(ctx, ":x: The value provided was not a valid number").await?; },
        Ok(delete_n) => {
            let mut find_msg = msg.channel_id
                .say(ctx, format!(":hourglass: Finding and deleting {} messages...", delete_n)).await?;

            let channel = &msg.channel(ctx).await.unwrap().guild().unwrap();

            let messages = &channel
                .messages(ctx, |r| r.before(&msg.id).limit(delete_n)).await?;
            let message_ids = messages.iter().map(|m| m.id).collect::<Vec<MessageId>>();

            channel.delete_messages(ctx, message_ids).await?;

            find_msg.edit(ctx, |m| {
                m.content(format!(":white_check_mark: Deleted {} messages", delete_n));
                m
            }).await?;
        }
    }

    Ok(())
}