use serenity::prelude::Context;
use crate::helpers::global_data::Database;
use std::env;
use mongodb::Collection;
use mongodb::bson::{doc, Document};
use serde::{Serialize, Deserialize};
use mongodb::options::FindOneAndReplaceOptions;

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseGuild {
    pub _id: i64,
    pub prefix: Option<String>,
    pub counting: Option<GuildCounting>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct GuildCounting {
    pub channel: i64,
    pub count: i64,
}

impl DatabaseGuild {
    pub(crate) async fn get_or_insert_new(ctx: &Context, _id: i64) -> DatabaseGuild {
        let get_result = DatabaseGuild::get(ctx, _id).await;

        if get_result == None {
            bson::from_document(
                DatabaseGuild::insert_or_replace(ctx, DatabaseGuild {
                    _id,
                    prefix: None,
                    counting: None
                }).await
            ).unwrap()
        } else {
            bson::from_document(get_result.unwrap()).unwrap()
        }
    }

    pub(crate) async fn get(ctx: &Context, _id: i64) -> Option<Document> {
        let document_id = doc! { "_id": _id };
        let document = DatabaseGuild::get_collection(ctx).await.find_one(document_id, None).await.unwrap();

        document
    }

    pub(crate) async fn insert_or_replace(ctx: &Context, database_guild: DatabaseGuild) -> Document {
        let new_document = bson::to_document(&database_guild).unwrap();

        let mut replace_options = FindOneAndReplaceOptions::default();
        replace_options.upsert = Some(true);

        let collection = DatabaseGuild::get_collection(ctx).await;
        // Find and replace the document and return it
        match collection.find_one_and_replace(doc! { "_id": database_guild._id }, new_document, replace_options).await.unwrap() {
            Some(document) => document,
            None => {
                collection.find_one(doc! { "_id": database_guild._id }, None).await.unwrap().unwrap()
            }
        }
    }

    pub(crate) async fn delete(ctx: &Context, id: i64) -> mongodb::error::Result<Option<Document>> {
        let document_id = doc! { "_id": id };
        DatabaseGuild::get_collection(ctx).await.find_one_and_delete(document_id, None).await
    }

    async fn get_collection(ctx: &Context) -> Collection {
        let mongo_database = env::var("MONGO_DATABASE").unwrap();
        let database = ctx.data.read().await.get::<Database>().unwrap().database(&mongo_database);

        database.collection("guilds")
    }
}