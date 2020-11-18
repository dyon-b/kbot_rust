use serenity::prelude::TypeMapKey;
use mongodb::Client;
use std::time::Instant;
use serenity::model::id::{ChannelId, GuildId};
use std::sync::Arc;
use dashmap::DashMap;

pub struct Database;
pub struct Uptime;
pub struct CountingCache;
pub struct PrefixCache;

impl TypeMapKey for Database {
    type Value = Client;
}

impl TypeMapKey for Uptime {
    type Value = Instant;
}

impl TypeMapKey for CountingCache {
    type Value = Arc<DashMap<ChannelId, i64>>;
}

impl TypeMapKey for PrefixCache {
    type Value = Arc<DashMap<GuildId, String>>;
}