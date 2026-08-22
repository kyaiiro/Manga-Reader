use super::MangaEntry;
use reqwest::Client;
use scraper::{Html, Selector};

pub async fn search(query : &str, limit : &str, rating : &str) -> anyhow::Result<Vec<MangaEntry>> {
    let resp = Client::new()
        .get("https://weebcentral.com/search/data")
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) manga-reader-project")
        .query(&[
            ("text", query),
            ("sort", "Best Match"),
            ("order", "Ascending"),
            ("official", "Any"),
            ("anime", "Any"),
            ("adult", rating),
            ("display_mode", "Full Display"),
        ])
        .send()
        .await?;

    let body = resp.text().await?;
    let mut entries = Vec::new();
    let max_results = limit.parse::<usize>().unwrap_or(usize::MAX);

    let document = Html::parse_document(&body);
    let article_selector = Selector::parse("article").expect("invalid selector: article");
    let title_link_selector = Selector::parse("a.line-clamp-1").expect("invalid selector: a.line-clamp-1");

    for article in document.select(&article_selector) {
        if entries.len() >= max_results {
            break;
        }
        let Some(title_el) = article.select(&title_link_selector).next() else { continue };
        let Some(href) = title_el.value().attr("href") else { continue };
        
        let id = href
            .split('/')
            .skip_while(|part| *part != "series")
            .nth(1)
            .map(|s| s.to_string())
            .unwrap_or_default();
        let title = href
            .split('/')
            .skip_while(|part| *part != "series")
            .nth(2)
            .map(|s| s.to_string())
            .unwrap_or_default()
            .replace("-", " ");
        
        let resp = Client::new()
            .get(href)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) manga-reader-project")
            .send()
            .await?;

        let body = resp.text().await?;

        let document = Html::parse_document(&body);
        let cover_url = document
            .select(&Selector::parse(r#"meta[property="og:image"]"#).unwrap())
            .next()
            .and_then(|m| m.value().attr("content"))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let li_selector = Selector::parse("li").unwrap();
        let strong_selector = Selector::parse("strong").unwrap();
        let p_selector = Selector::parse("p").unwrap();
        let a_selector = Selector::parse("a").unwrap();
        let span_selector = Selector::parse("span").unwrap();

        let mut desc = String::new();
        let mut status = String::new();
        let mut year = String::new();

        for li in document.select(&li_selector) {
            if let Some(strong) = li.select(&strong_selector).next() {
                let key = strong.text().collect::<String>().trim().to_string();

                if key.starts_with("Description") {
                    if let Some(p) = li.select(&p_selector).next() {
                        desc = p.text().collect::<String>().trim().to_string();
                    }
                } else if key.starts_with("Status") {
                    if let Some(a) = li.select(&a_selector).next() {
                        status = a.text().collect::<String>().trim().to_string();
                    }
                } else if key.starts_with("Released") {
                    if let Some(sp) = li.select(&span_selector).next() {
                        year = sp.text().collect::<String>().trim().to_string();
                    }
                }
            }
        }
        entries.push(MangaEntry {
            id,
            title,
            desc,
            status,
            year,
            cover_url,
        });
    }
    Ok(entries)
}