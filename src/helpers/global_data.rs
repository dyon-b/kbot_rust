use serenity::prelude::TypeMapKey;
use mongodb::Client;
use std::time::Instant;
use serenity::model::id::ChannelId;
use std::sync::Arc;
use dashmap::DashMap;

pub struct Database;
pub struct Uptime;
pub struct CountingCache;

impl TypeMapKey for Database {
    type Value = Client;
}

impl TypeMapKey for Uptime {
    type Value = Instant;
}

impl TypeMapKey for CountingCache {
    type Value = Arc<DashMap<ChannelId, i64>>;
}