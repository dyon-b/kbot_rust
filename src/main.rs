mod commands;
mod helpers;
mod models;

use std::{
    collections::HashSet,
    env,
    sync::Arc,
    time::Instant
};
use serenity::{async_trait, client::bridge::gateway::ShardManager, framework::{
    StandardFramework,
    standard::{
        help_commands,
        macros::{
            group,
            help,
            hook,
        }
    },
}, http::Http, model::{event::ResumedEvent, gateway::Ready}, prelude::*};
use tracing::{error, info};
use tracing_subscriber::{
    FmtSubscriber,
    EnvFilter,
};
use commands::{
    meta::*,
    moderation::*,
    configuration::*,
    aviation::*,
};

use helpers::global_data::Database;
use mongodb::Client as MongoClient;
use mongodb::options::ClientOptions as MongoClientOptions;

use serenity::client::bridge::gateway::GatewayIntents;
use serenity::framework::standard::{CommandResult, HelpOptions, Args, CommandGroup, CommandError, DispatchError};
use serenity::model::channel::Message;
use serenity::model::id::{UserId, ChannelId, GuildId};
use serenity::model::guild::{Guild, GuildUnavailable};
use crate::helpers::database_helper::DatabaseGuild;
use crate::helpers::global_data::{Uptime, CountingCache, PrefixCache, ReqwestContainer};
use serenity::futures::StreamExt;
use dashmap::DashMap;

use reqwest::Client as ReqwestClient;
use reqwest::redirect::Policy;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn guild_delete(&self, ctx: Context, _incomplete: GuildUnavailable, _full: Option<Guild>) {
        // Delete guild from database
        match DatabaseGuild::delete(&ctx, _incomplete.id.0 as i64).await {
            Ok(document) => {
                if document.is_some() {
                    // Remove from cache
                    let database_guild = bson::from_document::<DatabaseGuild>(document.unwrap()).unwrap();
                    if database_guild.counting.is_some() {
                        ctx.data.read().await.get::<CountingCache>().unwrap().remove(&ChannelId::from(database_guild.counting.unwrap().channel as u64));
                    }
                    if database_guild.prefix.is_some() {
                        ctx.data.read().await.get::<PrefixCache>().unwrap().remove(&_incomplete.id);
                    }
                }
            },
            Err(why) => error!("Error when deleting guild from database: {}", why),
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let counting_cache = ctx.data.read().await.get::<CountingCache>().cloned().unwrap();

        // Counting channel
        if let Some(counting_map) = counting_cache.get(&msg.channel_id) {
            if let Ok(new_number) = msg.content.parse::<i64>() {
                if new_number == counting_map.value() + 1 {
                    counting_cache.insert(msg.channel_id, new_number);

                    // Edit the database to show the right number
                    let mut database_guild = DatabaseGuild::get_or_insert_new(&ctx, msg.guild_id.unwrap().0 as i64).await;
                    // Stupid hacky stuff
                    let mut new_counting = database_guild.counting.unwrap();
                    new_counting.count += 1;
                    database_guild.counting.replace(new_counting);

                    DatabaseGuild::insert_or_replace(&ctx, database_guild).await;
                } else {
                    // Delete the message if it's not the correct number
                    match msg.delete(&ctx).await {
                        _ => {}
                    };
                }
            } else {
                // Delete the message if it couldn't parse as a number
                match msg.delete(&ctx).await {
                    _ => {}
                };
            }
        };
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(ping, about, invite, serverinfo)]
struct Meta;

#[group]
#[commands(purge)]
struct Moderation;

#[group]
#[prefixes("config", "configure", "conf")]
#[commands(prefix, count)]
struct Configuration;

#[group]
#[commands(icao)]
struct Aviation;

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
#[embed_error_colour(RED)]
#[embed_success_colour(BLURPLE)]
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

// This is for errors that happen before command execution.
#[hook]
async fn on_dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::NotEnoughArguments { min, given } => {
            let error_string = {
                if given == 0 && min == 1 {
                    format!(":no_entry_sign: I need an argument to run this command.")
                } else if given == 0 {
                    format!(":no_entry_sign: I need at least {} arguments to run this command.", min)
                } else {
                    format!(":no_entry_sign: I need {} arguments to run this command, But I was only given {}.", min, given)
                }
            };
            let _ = msg.channel_id.say(ctx, error_string).await;
        }
        DispatchError::TooManyArguments { max, given } => {
            let _ = msg.channel_id.say(ctx, format!(":no_entry_sign: I needed a maximum of {} argument(s) but you gave me {}.", max, given)).await;
        }
        DispatchError::Ratelimited(x) => {
            let _ = msg.reply(ctx, format!(":no_entry_sign: You can use this command again in {} seconds.", x.as_secs())).await;
        }
        DispatchError::LackingPermissions(permissions) => {
            let _ = msg.channel_id.say(ctx, format!(":no_entry_sign: You're lacking these permissions to run this command: `{}`", permissions)).await;
        }
        DispatchError::OnlyForGuilds => { let _ = msg.channel_id.say(ctx, ":no_entry_sign: This command is for guilds only.").await; }
        DispatchError::OnlyForDM => { let _ = msg.channel_id.say(ctx, ":no_entry_sign: This command is for direct messages only.").await; }
        _ => {
            error!("Unhandled dispatch error: {:?}", error);
        }
    }
}

#[hook]
async fn after(_: &Context, _: &Message, command_name: &str, error: Result<(), CommandError>) {
    if let Err(why) = error {
        error!("Error in {}: {:?}", command_name, why);
    }
}

#[hook]
async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> { // Custom per guild prefixes.
    let guild_id = &msg.guild_id;

    if guild_id.is_some() {
        let prefix_cache = ctx.data.read().await.get::<PrefixCache>().cloned().unwrap();

        // Counting channel
        if let Some(prefix_map) = prefix_cache.get(&guild_id.unwrap()) {
            return Some(prefix_map.value().parse().unwrap());
        };
    }

    Some(env::var("DEFAULT_PREFIX").unwrap_or_else(|_| String::from("?")))
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

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|config| config
            .owners(owners)
            .on_mention(Some(_bot_id))
            // Disable default prefix
            .prefix("")
            .dynamic_prefix(dynamic_prefix)
            .allow_dm(true)
            .ignore_bots(true)
            .ignore_webhooks(true)
        )
        .on_dispatch_error(on_dispatch_error)
        .after(after)
        .group(&META_GROUP)
        .group(&MODERATION_GROUP)
        .group(&CONFIGURATION_GROUP)
        .group(&AVIATION_GROUP)
        .help(&MY_HELP);

    //let avwx_token = env::var("AVWX_TOKEN").unwrap();

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .intents(
            GatewayIntents::GUILD_MESSAGES |
            GatewayIntents::GUILDS |
            GatewayIntents::DIRECT_MESSAGES |
            GatewayIntents::GUILD_MESSAGE_REACTIONS |
            GatewayIntents::DIRECT_MESSAGE_REACTIONS |
            GatewayIntents::GUILD_PRESENCES
        ).await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());

        let mongo_database = env::var("MONGO_DATABASE").unwrap();
        // Mongo client options
        let connection_url = env::var("MONGO_URL").unwrap_or_else(|_| String::from("mongodb://127.0.0.1:27017"));
        let mut client_options = match MongoClientOptions::parse(&connection_url).await {
            Ok(options) => options,
            Err(why) => panic!("Error occurred getting mongo client options: {}", why),
        };
        client_options.app_name = Some("kbot_rust".to_string());
        // Store mongo client in context data
        let mongo_client = match MongoClient::with_options(client_options) {
            Ok(client) => client,
            Err(why) => panic!("Error occurred getting mongo client: {}", why),
        };
        data.insert::<Database>(mongo_client);

        let counting_cache: DashMap<ChannelId, i64> = DashMap::new();
        let prefix_cache: DashMap<GuildId, String> = DashMap::new();
        // Iterate through every guild in the database
        let mut database_guilds_cursor = data.get::<Database>().unwrap().database(&mongo_database).collection("guilds").find(None, None).await.unwrap();
        while let Some(document) = database_guilds_cursor.next().await {
            let database_guild = bson::from_document::<DatabaseGuild>(document.unwrap()).unwrap();
            if database_guild.prefix.is_some() {
                prefix_cache.insert(GuildId::from(database_guild._id as u64), database_guild.prefix.unwrap());
            }
            if database_guild.counting.is_some() {
                counting_cache.insert(ChannelId::from(database_guild.counting.unwrap().channel as u64), database_guild.counting.unwrap().count);
            }
        }

        // Insert the DashMaps
        data.insert::<CountingCache>(Arc::from(counting_cache));
        data.insert::<PrefixCache>(Arc::from(prefix_cache));

        // Insert uptime to global data
        data.insert::<Uptime>(Instant::now());

        // Insert a new Reqwest client
        data.insert::<ReqwestContainer>(ReqwestClient::builder().redirect(Policy::none()).build().unwrap());
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}
