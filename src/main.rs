mod commands;

use std::{
    collections::HashSet,
    env,
    sync::Arc,
};
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{
        StandardFramework,
        standard::{
            help_commands,
            macros::{
                group,
                help,
            }
        },
    },
    http::Http,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};
use tracing::{error, info};
use tracing_subscriber::{
    FmtSubscriber,
    EnvFilter,
};
use commands::{
    other::*,
};
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::framework::standard::{CommandResult, HelpOptions, Args, CommandGroup};
use serenity::model::channel::Message;
use serenity::model::id::UserId;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(ping, about)]
struct General;

#[help]
#[individual_command_tip =
"Hello!
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name
// and commands.
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    dotenv::dotenv().expect("Failed to load .env file");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let http = Http::new_with_token(&token);

    // Fetch owners id and bot id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let default_prefix = env::var("DEFAULT_PREFIX")
        .expect("!");
    // Create the framework
    let framework = StandardFramework::new()
        .configure(|config| config
            .owners(owners)
            .on_mention(Some(_bot_id))
            .prefix(&default_prefix)
            .allow_dm(false)
            .ignore_bots(true)
            .ignore_webhooks(true)
        )
        .group(&GENERAL_GROUP)
        .help(&MY_HELP);

    let mut client = Client::new(&token)
        .framework(framework)
        .event_handler(Handler)
        .add_intent(GatewayIntents::GUILD_MESSAGES)
        .add_intent(GatewayIntents::GUILDS)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
