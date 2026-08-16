use super::MangaEntry;

pub fn search(query : &str, limit : &str, rating : &str) -> Vec<MangaEntry> {
    let client = reqwest::blocking::Client::new();
    let resp_json = match client
        .get("https://api.mangadex.org/manga")
        .header("User-Agent", "manga-reader-project/0.1")
        .query(&[
            ("title", query),
            ("limit", limit),
            ("contentRating[]", "safe"),
            ("contentRating[]", rating),
            ("includes[]", "cover_art"),
        ])
        .send()
    {
        Ok(resp) => match resp.json::<serde_json::Value>() {
            Ok(j) => j,
            Err(_) => return Vec::new(),
        },
        Err(_) => return Vec::new(),
    };

    let items : &[serde_json::Value] = resp_json["data"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    items
        .iter()
        .map(|item| {
            let id = item["id"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let title = item["attributes"]["title"]
                .as_object()
                .and_then(|obj| obj.values().next())
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let desc = item["attributes"]["description"]["en"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let status = item["attributes"]["status"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let year = item["attributes"]["year"]
                .as_i64()
                .map(|v| v.to_string())
                .or_else(|| item["attributes"]["year"].as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            let cover_url = item["relationships"]
                .as_array()
                .and_then(|relationships| {
                    relationships.iter().find(|rel| rel["type"].as_str() == Some("cover_art"))
                })
                .and_then(|rel| rel["attributes"]["fileName"].as_str())
                .map(|file_name| format!("https://uploads.mangadex.org/covers/{}/{}", id, file_name))
                .unwrap_or_default();

            MangaEntry {
                id,
                title,
                desc,
                status,
                year,
                cover_url,
            }
        })
        .collect()
}