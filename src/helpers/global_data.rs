use serenity::prelude::TypeMapKey;
use mongodb::Client;

pub struct Database;

impl TypeMapKey for Database {
    type Value = Client;
}