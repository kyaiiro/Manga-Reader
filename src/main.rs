mod sources;
mod shortcuts;
use shortcuts::{cover, md};
use sources::{MangaEntry, mangadex, weebcentral, dynasty};

use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use eframe::egui::{self};
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

#[derive(PartialEq)]
enum Page {
    Home,
    Search,
    Details,
}

struct MangaReaderApp {
    page: Page,
    search_text: String,
    entries: Vec<MangaEntry>,
    textures: HashMap<String, egui::TextureHandle>,
    loading: std::collections::HashSet<String>,
    image_tx: mpsc::Sender<(String, Vec<u8>)>,
    image_rx: mpsc::Receiver<(String, Vec<u8>)>,
    search_tx: mpsc::Sender<Vec<MangaEntry>>,
    search_rx: mpsc::Receiver<Vec<MangaEntry>>,
    searching: bool,
    selected_entry: Option<MangaEntry>,
    md_cache: CommonMarkCache,
}

impl Default for MangaReaderApp {
    fn default() -> Self {
        let (image_tx, image_rx) = mpsc::channel::<(String, Vec<u8>)>();
        let (search_tx, search_rx) = mpsc::channel::<Vec<MangaEntry>>();
        Self {
            page: Page::Home,
            search_text: String::new(),
            entries: Vec::new(),
            textures: HashMap::new(),
            loading: std::collections::HashSet::new(),
            image_tx,
            image_rx,
            search_tx,
            search_rx,
            searching: false,
            selected_entry: None,
            md_cache: CommonMarkCache::default(),
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

        while let Ok(results) = self.search_rx.try_recv() {
            self.entries = results;
            self.searching = false;
        }

        egui::Panel::top("nav_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.page == Page::Home, "Home").clicked() {
                    self.selected_entry = None;
                    self.page = Page::Home;
                }
                if ui.selectable_label(self.page == Page::Search, "Search").clicked() {
                    self.selected_entry = None;
                    self.page = Page::Search;
                }
            });
        });

        match self.page {
            Page::Home => self.show_home(ui),
            Page::Search => self.show_search(ui),
            Page::Details => self.show_details(ui),
        }

        if self.selected_entry.is_some() {
            self.page = Page::Details;
        }
    }
}

impl MangaReaderApp {
    fn show_home(&mut self, ui: &mut egui::Ui) {
        
    }

    fn show_search(&mut self, ui: &mut egui::Ui) {
        let mut clicked: Option<usize> = None;

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                let query = ui.add(egui::TextEdit::singleline(&mut self.search_text).hint_text("Search for manga..."));

                if query.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.entries.clear();
                    self.textures.clear();
                    self.loading.clear();
                    self.searching = true;

                    let query = self.search_text.clone();
                    let tx = self.search_tx.clone();
                    let ctx = ui.ctx().clone();
                    std::thread::spawn(move || {
                        let results = mangadex::search(&query, "4", "safe");
                        let _ = tx.send(results);
                        ctx.request_repaint();
                    });
                }

                ui.label(egui::RichText::from("MangaDex")
                    .heading()
                    .size(32.0));
                if self.entries.is_empty() && !self.searching {
                    ui.label("No results yet, search for something...");
                }

                ui.horizontal(|ui| {
                    for (idx, item) in self.entries.iter().enumerate() {
                        let url = item.cover_url.clone() +".256.jpg";
                        ui.vertical(|ui| {
                            ui.set_max_width(256.0);
                            if let Some(tex) = self.textures.get(&url) {
                                let image = egui::Image::from_texture((tex.id(), egui::Vec2::new(256.0, 336.0)))
                                    .sense(egui::Sense::click());
                                let response = ui.add(image);
                                if response.clicked() {
                                    clicked = Some(idx);
                                }
                            } else {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(256.0, 336.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, 0.0, egui::Color32::DARK_GRAY);
                            }
                            ui.add(egui::Label::new(egui::RichText::from(&item.title).heading()).truncate());
                        });
                    }
                });

                if self.searching {
                    ui.label(egui::RichText::from("Searching...").size(16.0));
                }
            });
        });

        if let Some(idx) = clicked {
            self.selected_entry = Some(self.entries[idx].clone());
        }

        for item in &self.entries {
            let url = item.cover_url.clone() + ".256.jpg";
            if !self.textures.contains_key(&url) && !self.loading.contains(&url) {
                self.loading.insert(url.clone());
                cover::load(url, self.image_tx.clone(), ui.ctx().clone());
            }
        }
    }

    fn show_details(&mut self, ui: &mut egui::Ui) {
        let entry = self.selected_entry.clone().unwrap();
        let url = entry.cover_url.clone() + ".256.jpg";
        let title = entry.title.clone();
        let desc = entry.desc.clone();
        //TODO Add chapters

        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if let Some(tex) = self.textures.get(&url) {
                        ui.image((tex.id(), egui::Vec2::new(256.0, 336.0)));
                    } else {
                        let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(256.0, 336.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 4.0, egui::Color32::DARK_GRAY);
                    }
                    //TODO add a button here to add the entry to your home page. Save these entries in a json or smt?

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::from(title)
                            .heading()
                            .size(34.0)
                            .color(egui::Color32::WHITE));
                        ui.add_space(6.0);
                        egui::ScrollArea::vertical()
                            .max_height(336.0 - 40.0)
                            .id_salt("desc_scroll")
                            .show(ui, |ui| {
                                ui.scope(|ui| {
                                    ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);

                                    let style = ui.style_mut();
                                    style.text_styles.insert(
                                        egui::TextStyle::Body,
                                        egui::FontId::new(24.0, egui::FontFamily::Proportional),
                                    );

                                    CommonMarkViewer::new().show(ui, &mut self.md_cache, &md::normalize_markdown(&desc));
                                });
                            });
                    })
                });
            });
        });
    }
}