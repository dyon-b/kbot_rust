use serenity::prelude::TypeMapKey;
use mongodb::Client;
use std::time::Instant;

pub struct Database;
pub struct Uptime;

impl TypeMapKey for Database {
    type Value = Client;
}

impl TypeMapKey for Uptime {
    type Value = Instant;
}