use serenity::prelude::TypeMapKey;
use mongodb::Client as MongoClient;
use std::time::Instant;
use serenity::model::id::{ChannelId, GuildId};
use std::sync::Arc;
use dashmap::DashMap;
use reqwest::Client as ReqwestClient;

pub struct Database;
pub struct Uptime;
pub struct CountingCache;
pub struct PrefixCache;
pub struct ReqwestContainer;

impl TypeMapKey for Database {
    type Value = MongoClient;
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

impl TypeMapKey for ReqwestContainer {
    type Value = ReqwestClient;
}