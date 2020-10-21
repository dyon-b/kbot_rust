use serenity::prelude::Context;
use crate::helpers::global_data::Database;
use std::env;
use mongodb::Collection;
use mongodb::bson::doc;

pub struct DatabaseGuild {
    pub id: u64,
    pub prefix: String,
}

impl DatabaseGuild {
    pub(crate) async fn get(ctx: &Context, id: u64) {
        let document_id = doc! { "_id": id };
        let document = DatabaseGuild::get_collection(ctx).await.find_one(document_id, None).await.unwrap();

        println!("{:?}", document);
    }

    async fn delete(&self, ctx: &Context) {
        let document_id = doc! { "_id": self.id };
        DatabaseGuild::get_collection(ctx).await.find_one_and_delete(document_id, None).await;
    }

    async fn get_collection(ctx: &Context) -> Collection {
        let mongo_database = env::var("MONGO_DATABASE").unwrap();
        let database = ctx.data.read().await.get::<Database>().unwrap().database(&mongo_database);

        database.collection("guilds")
    }
}