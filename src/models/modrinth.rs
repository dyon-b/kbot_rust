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

impl From<FullModrinthMod> for SearchedModrinthMod {
    fn from(full_mod: FullModrinthMod) -> Self {
        Self {
            mod_id: full_mod.id,
            author: "author".to_string(),
            title: full_mod.title,
            description: full_mod.description,
            categories: full_mod.categories,
            versions: full_mod.versions,
            downloads: full_mod.downloads,
            page_url: "page_url".to_string(),
            icon_url: full_mod.icon_url,
            author_url: "author_url".to_string(),
            date_created: full_mod.published,
            date_modified: full_mod.updated,
            latest_version: "latest_version".to_string(),
            host: "host".to_string()
        }
    }
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