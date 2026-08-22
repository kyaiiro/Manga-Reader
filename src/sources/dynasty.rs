use super::MangaEntry;
use reqwest::Client;
use scraper::{Html, Selector};

pub async fn search(query: &str, limit: &str) -> anyhow::Result<Vec<MangaEntry>> {
    let resp = Client::new()
        .get("https://dynasty-scans.com/search")
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) manga-reader-project")
        .query(&[
            ("q", query),
            ("classes[]", "Series"),
            ("sort", ""),
        ])
        .send()
        .await?;

    let body = resp.text().await?;
    let document = Html::parse_document(&body);
    let result_selector = Selector::parse("dl.chapter-list dd a.name").expect("invalid selector");
    let detail_cover_selector = Selector::parse("div.cover img.thumbnail").expect("invalid selector");
    let detail_desc_selector = Selector::parse("div.description p").expect("invalid selector");
    let detail_status_selector = Selector::parse("h2.tag-title small").expect("invalid selector");
    let chapter_date_selector = Selector::parse("dl.chapter-list dd small").expect("invalid selector");

    let max_results = limit.parse::<usize>().unwrap_or(usize::MAX);
    let mut entries = Vec::new();

    for element in document.select(&result_selector) {
        if entries.len() >= max_results {
            break;
        }

        let href = match element.value().attr("href") {
            Some(h) => h,
            None => continue,
        };

        let title = element.text().collect::<String>().trim().to_string();
        let id = href
            .trim_start_matches("/series/")
            .trim_start_matches('/')
            .to_string();

        let detail_url = if href.starts_with("http") {
            href.to_string()
        } else if href.starts_with('/') {
            format!("https://dynasty-scans.com{}", href)
        } else {
            format!("https://dynasty-scans.com/{}", href)
        };

        let detail_resp = Client::new()
            .get(&detail_url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) manga-reader-project")
            .send()
            .await?;

        let detail_body = detail_resp.text().await?;
        let detail_doc = Html::parse_document(&detail_body);

        let cover_url = detail_doc
            .select(&detail_cover_selector)
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(|src| {
                if src.starts_with("http") {
                    src.to_string()
                } else if src.starts_with('/') {
                    format!("https://dynasty-scans.com{}", src)
                } else {
                    format!("https://dynasty-scans.com/{}", src)
                }
            })
            .unwrap_or_default();

        let status = detail_doc
            .select(&detail_status_selector)
            .next()
            .map(|small| {
                let raw = small.text().collect::<String>().trim().to_string();
                raw.strip_prefix('—')
                    .or_else(|| raw.strip_prefix("-"))
                    .unwrap_or(&raw)
                    .trim()
                    .to_string()
            })
            .unwrap_or_default();

        let desc = detail_doc
            .select(&detail_desc_selector)
            .filter_map(|p| {
                let text = p.text().collect::<String>().trim().to_string();
                if text.is_empty() || text.starts_with("Related:") {
                    None
                } else {
                    Some(text)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let year = detail_doc
            .select(&chapter_date_selector)
            .next()
            .and_then(|small| {
                let text = small.text().collect::<String>();
                text.rsplit('\'')
                    .next()
                    .and_then(|suffix| {
                        if suffix.len() == 2 && suffix.chars().all(char::is_numeric) {
                            suffix.parse::<u32>().ok().map(|yy| {
                                if yy >= 90 {
                                    format!("19{:02}", yy)
                                } else {
                                    format!("20{:02}", yy)
                                }
                            })
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or_default();

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
