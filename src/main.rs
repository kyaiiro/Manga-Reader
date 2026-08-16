mod sources;
use sources::MangaEntry;
#[allow(unused)]
use sources::{mangadex, weebcentral, dynasty};
// fn main() {
//     let entries = mangadex::search("I want to love you till", "4", "safe");
//     for item in entries {
//         println!("{:#?}", item.title);
//     }
//     let entries = weebcentral::search("", "10", "Any");
//     println!("{:#?}", entries);
//     let entries = dynasty::search("", "10");
//     println!("{:#?}", entries);
// }

use eframe::egui::self;
use std::sync::mpsc;
use std::collections::HashMap;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Manga Reader App",
        native_options,
        Box::new(|_cc| Ok(Box::new(MangaReaderApp::default()))),
    )
}

#[allow(unused)]
struct MangaReaderApp {
    search_text: String,
    entries: Vec<MangaEntry>,
    textures: HashMap<String, egui::TextureHandle>,
    loading: std::collections::HashSet<String>,
    image_tx: mpsc::Sender<(String, Vec<u8>)>,
    image_rx: mpsc::Receiver<(String, Vec<u8>)>,
}

impl Default for MangaReaderApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel::<(String, Vec<u8>)>();
        Self {
            search_text: String::new(),
            entries: Vec::new(),
            textures: HashMap::new(),
            loading: std::collections::HashSet::new(),
            image_tx: tx,
            image_rx: rx,
        }
    }
}

impl eframe::App for MangaReaderApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Ok((url, bytes)) = self.image_rx.try_recv() {
            self.loading.remove(&url);
            
            if bytes.is_empty() {
                continue;
            }
            if let Ok(img) = image::load_from_memory(&bytes) {
                let img = img.to_rgba8();
                let (w, h) = img.dimensions();
                let pixels = img.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);
                let texture = ui.ctx().load_texture(&url, color_image, egui::TextureOptions::default());
                self.textures.insert(url, texture);
                ui.ctx().request_repaint();
            }
        }
        
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                let query = ui.add(egui::TextEdit::singleline(&mut self.search_text).hint_text("Search for manga..."));

                if query.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.entries.clear();
                    self.textures.clear();
                    self.loading.clear();

                    self.entries = mangadex::search(&self.search_text, "4", "safe");
                }

                ui.horizontal(|ui| {
                    for item in &self.entries {
                        let url = item.cover_url.clone() + ".128.jpg";
                        println!("{}", url);
                        if let Some(tex) = self.textures.get(&url) {
                            ui.vertical(|ui| {
                                ui.image((tex.id(), egui::Vec2::new(128.0, 192.0)));
                                ui.label("");
                            });
                        } else {
                            ui.label("Loading...");
                        }
                    }
                });
            });
        });

        for item in &self.entries {
            let url = item.cover_url.clone() + ".128.jpg";
            if !self.textures.contains_key(&url) && !self.loading.contains(&url) {
                self.loading.insert(url.clone());
                let tx = self.image_tx.clone();
                let url_clone = url.clone();
                std::thread::spawn(move || {
                    let client = reqwest::blocking::Client::builder()
                        .user_agent("MangaReaderApp/0.1")
                        .build()
                        .unwrap();

                    let bytes = match client.get(&url_clone).send() {
                        Ok(resp) => {
                            let status = resp.status();
                            if !status.is_success() {
                                let body = resp.text().unwrap_or_default();
                                eprintln!("fetch failed for {url_clone}: HTTP {status} - {body}");
                                Vec::new()
                            } else {
                                resp.bytes().map(|b| b.to_vec()).unwrap_or_default()
                            }
                        }
                        Err(e) => {
                            eprintln!("fetch error for {url_clone}: {e}");
                            Vec::new()
                        }
                    };
                    let _ = tx.send((url_clone, bytes));
                });
            }
        }

    }
}

//TODO Names of the results under the image