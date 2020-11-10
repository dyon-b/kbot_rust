use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ModrinthModSearch {
    pub hits: Vec<ModrinthMod>,
    pub offset: i32,
    pub limit: i32,
    pub total_hits: i32,
}

#[derive(Deserialize, Debug)]
pub struct ModrinthMod {
    pub mod_id: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub downloads: i32,
    pub page_url: String,
    pub icon_url: String,
    pub author_url: String,
    pub date_created: String,
    pub date_modified: String,
    pub latest_version: String,
    pub host: String,
}