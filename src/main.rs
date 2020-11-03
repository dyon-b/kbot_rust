mod commands;
mod helpers;

use std::{
    collections::HashSet,
    env,
    sync::Arc,
    time::Instant
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
                hook,
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
    meta::*,
    moderation::*,
    info::*,
    configuration::*,
    modrinth::*,
};

use helpers::global_data::Database;
use mongodb::Client as MongoClient;
use mongodb::options::ClientOptions as MongoClientOptions;

use serenity::client::bridge::gateway::GatewayIntents;
use serenity::framework::standard::{CommandResult, HelpOptions, Args, CommandGroup, CommandError};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::model::guild::{Guild, GuildUnavailable};
use crate::helpers::database_helper::DatabaseGuild;
use crate::helpers::global_data::Uptime;

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
            Ok(_) => {},
            Err(why) => error!("Error when deleting guild from database: {}", why),
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(ping, about)]
struct Meta;

#[group]
#[commands(purge)]
struct Moderation;

#[group]
#[commands(serverinfo)]
struct Info;

#[group]
#[prefixes("config", "configure", "conf")]
#[commands(prefix)]
struct Configuration;

#[group]
#[prefixes("mr", "modrinth")]
#[commands(search)]
struct Modrinth;

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

#[hook]
async fn after(_: &Context, _: &Message, command_name: &str, error: Result<(), CommandError>) {
    if let Err(why) = error {
        error!("Error in {}: {:?}", command_name, why);
    }
}

#[hook]
async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> { // Custom per guild prefixes.
    let guild_id = &msg.guild_id;

    if let Some(_) = guild_id {
        // This looks horrible
        match DatabaseGuild::get(ctx, guild_id.unwrap().0 as i64).await {
            Some(document) => {
                let database_guild: DatabaseGuild = bson::from_document(document).unwrap();
                match database_guild.prefix {
                    Some(new_prefix) => return Some(new_prefix),
                    None => {},
                }
            },
            None => {},
        }
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
            //.prefix(&default_prefix)
            .dynamic_prefix(dynamic_prefix)
            .allow_dm(true)
            .ignore_bots(true)
            .ignore_webhooks(true)
        )
        .after(after)
        .group(&META_GROUP)
        .group(&MODERATION_GROUP)
        .group(&INFO_GROUP)
        .group(&CONFIGURATION_GROUP)
        .group(&MODRINTH_GROUP)
        .help(&MY_HELP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .add_intent(GatewayIntents::GUILD_MESSAGES)
        .add_intent(GatewayIntents::GUILDS)
        .add_intent(GatewayIntents::DIRECT_MESSAGES)
        .add_intent(GatewayIntents::GUILD_MESSAGE_REACTIONS)
        .add_intent(GatewayIntents::DIRECT_MESSAGE_REACTIONS)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());

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

        // Insert uptime to global data
        data.insert::<Uptime>(Instant::now());
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}
