use serde::{Serialize, Deserialize};
pub mod mangadex;
pub mod weebcentral;
pub mod dynasty;

#[allow(unused)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MangaEntry {
    pub id: String,
    pub title: String,
    pub desc: String,
    pub status: String,
    pub year: String,
    pub cover_url: String,
}