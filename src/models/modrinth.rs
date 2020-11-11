use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ModrinthModSearch {
    pub hits: Vec<SearchedModrinthMod>,
    pub offset: i32,
    pub limit: i32,
    pub total_hits: i32,
}

#[derive(Deserialize, Debug)]
pub struct SearchedModrinthMod {
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

#[derive(Deserialize, Debug)]
pub struct FullModrinthMod {
    pub id: String,
    pub team: String,
    pub title: String,
    pub description: String,
    pub body_url: String,
    pub published: String,
    pub updated: String,
    pub status: String,
    pub downloads: i32,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub icon_url: String,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
}