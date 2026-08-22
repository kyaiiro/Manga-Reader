use std::sync::mpsc;
use eframe::egui::self;

pub fn load(url: String, tx: mpsc::Sender<(String, Vec<u8>)>, ctx: egui::Context) {
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .user_agent("MangaReaderApp/0.1")
            .build()
            .unwrap();

        let bytes = match client.get(&url).send() {
            Ok(resp) => {
                let status = resp.status();
                if !status.is_success() {
                    let body = resp.text().unwrap_or_default();
                    eprintln!("fetch failed for {url}: HTTP {status} - {body}");
                    Vec::new()
                } else {
                    resp.bytes().map(|b| b.to_vec()).unwrap_or_default()
                }
            }
            Err(e) => {
                eprintln!("fetch error for {url}: {e}");
                Vec::new()
            }
        };

        let _ = tx.send((url, bytes));
        ctx.request_repaint();
    });
}