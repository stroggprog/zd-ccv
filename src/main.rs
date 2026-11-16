mod card;

use card::{CardFile, FieldValue};
use clap::Parser;
use eframe::egui::{self, Align, Layout, RichText, ScrollArea, Sense, Vec2};
use std::collections::HashMap;
use std::path::PathBuf;

/// A cross-platform desktop viewer for SillyTavern character cards.
#[derive(Parser)]
#[command(name = "zd-ccv", version, about)]
struct Args {
    /// Card PNG files or folders to open; only `.png` files are loaded
    /// (folders contribute their direct files, subfolders are not
    /// traversed). Without arguments a folder picker opens at startup.
    paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme {
    System,
    Light,
    Dark,
}

impl Theme {
    const ALL: [Theme; 3] = [Theme::System, Theme::Light, Theme::Dark];

    fn label(self) -> &'static str {
        match self {
            Theme::System => "System",
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        }
    }

    fn from_label(label: &str) -> Option<Theme> {
        Theme::ALL.into_iter().find(|t| t.label() == label)
    }
}

#[derive(Debug, Clone, Copy)]
enum PendingDialog {
    Files,
    Folder,
}

/// Frames during which the data panel is pinned to the top after a card
/// load or chunk switch, so that residual smoothed wheel input (which egui
/// keeps feeding for several frames after the user scrolls) cannot scroll
/// the freshly loaded tall card away from the top. The wheel momentum
/// decay is far past its tail after this budget.
const SCROLL_PIN_FRAMES: u8 = 30;

struct Viewer {
    files: Vec<PathBuf>,
    index: usize,
    card: Option<CardFile>,
    textures: HashMap<PathBuf, egui::TextureHandle>,
    /// Note about the last texture (decode error); keyed by path.
    texture_errors: HashMap<PathBuf, String>,
    theme: Theme,
    system_dark: bool,
    applied_dark: Option<bool>,
    zoom: f32,
    pan: Vec2,
    /// Index of the embedded chunk currently displayed (see CardFile.payloads).
    display_chunk: usize,
    /// Remaining frames of the data-panel top pin (SCROLL_PIN_FRAMES).
    scroll_pin: u8,
    #[cfg(test)]
    last_scroll_id: Option<egui::Id>,
    pending_dialog: Option<PendingDialog>,
    startup_dialog_done: bool,
}

impl Viewer {
    fn new(cc: &eframe::CreationContext<'_>, files: Vec<PathBuf>) -> Self {
        let theme = cc
            .storage
            .and_then(|s| eframe::get_value::<String>(s, "theme"))
            .and_then(|s| Theme::from_label(&s))
            .unwrap_or(Theme::System);
        let system_dark = detect_system_dark();

        let mut viewer = Viewer {
            files,
            index: 0,
            card: None,
            textures: HashMap::new(),
            texture_errors: HashMap::new(),
            theme,
            system_dark,
            applied_dark: None,
            zoom: 1.0,
            pan: Vec2::ZERO,
            display_chunk: 0,
            scroll_pin: 0,
            #[cfg(test)]
            last_scroll_id: None,
            pending_dialog: None,
            startup_dialog_done: false,
        };
        viewer.reload_card();
        viewer
    }

    fn reload_card(&mut self) {
        self.card = self.files.get(self.index).map(|p| card::load(p));
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
        self.display_chunk = 0;
        self.scroll_pin = SCROLL_PIN_FRAMES;
    }

    fn set_files(&mut self, mut files: Vec<PathBuf>) {
        files.sort();
        self.files = files;
        self.index = 0;
        self.reload_card();
    }

    fn file_name(&self) -> String {
        self.files
            .get(self.index)
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "no card loaded".into())
    }

    fn effective_dark(&self) -> bool {
        match self.theme {
            Theme::Light => false,
            Theme::Dark => true,
            Theme::System => self.system_dark,
        }
    }

    fn step(&mut self, delta: isize) {
        if self.files.is_empty() {
            return;
        }
        let next = self.index as isize + delta;
        if next < 0 || next >= self.files.len() as isize {
            return;
        }
        self.index = next as usize;
        self.reload_card();
    }

    /// Texture for the current card, decoding and uploading it lazily.
    fn texture_for(&mut self, ui: &mut egui::Ui) -> Option<egui::TextureHandle> {
        let path = self.files.get(self.index)?.clone();
        if let Some(tex) = self.textures.get(&path) {
            return Some(tex.clone());
        }
        if self.texture_errors.contains_key(&path) {
            return None;
        }
        match decode_to_color_image(&path) {
            Ok((color_image, name)) => {
                let tex = ui
                    .ctx()
                    .load_texture(name, color_image, egui::TextureOptions::LINEAR);
                self.textures.insert(path, tex.clone());
                Some(tex)
            }
            Err(e) => {
                self.texture_errors.insert(path, e);
                None
            }
        }
    }
}

fn decode_to_color_image(path: &std::path::Path) -> Result<(egui::ColorImage, String), String> {
    let img = image::ImageReader::open(path)
        .map_err(|e| format!("could not open image: {e}"))?
        .with_guessed_format()
        .map_err(|e| format!("could not read image: {e}"))?
        .decode()
        .map_err(|e| format!("could not decode image: {e}"))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut size_ok = (w as usize, h as usize);
    const MAX_SIDE: u32 = 8192;
    let rgba = if w > MAX_SIDE || h > MAX_SIDE {
        let scale = MAX_SIDE as f32 / w.max(h) as f32;
        let nw = ((w as f32 * scale) as u32).max(1);
        let nh = ((h as f32 * scale) as u32).max(1);
        size_ok = (nw as usize, nh as usize);
        image::imageops::resize(&rgba, nw, nh, image::imageops::FilterType::Triangle)
    } else {
        rgba
    };
    let color =
        egui::ColorImage::from_rgba_unmultiplied(size_ok.into(), rgba.as_flat_samples().as_slice());
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "texture".into());
    Ok((color, name))
}

fn detect_system_dark() -> bool {
    match dark_light::detect() {
        Ok(dark_light::Mode::Dark) => true,
        Ok(dark_light::Mode::Light) => false,
        // Unspecified or undetectable: assume dark.
        _ => true,
    }
}

impl eframe::App for Viewer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "theme", &self.theme.label().to_string());
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !self.startup_dialog_done {
            self.startup_dialog_done = true;
            if self.files.is_empty() {
                self.pending_dialog = Some(PendingDialog::Folder);
            }
        }
        apply_theme(ui.ctx(), self);

        let mut dialog: Option<PendingDialog> = None;
        egui::Panel::top("menubar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Files...").clicked() {
                        dialog = Some(PendingDialog::Files);
                        ui.close();
                    }
                    if ui.button("Open Folder...").clicked() {
                        dialog = Some(PendingDialog::Folder);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        self.toolbar(ui);
        self.data_panel(ui);
        self.image_panel(ui);

        if self.pending_dialog.is_none() {
            self.pending_dialog = dialog;
        }
        if let Some(kind) = self.pending_dialog.take() {
            match kind {
                PendingDialog::Files => {
                    if let Some(files) = rfd::FileDialog::new()
                        .set_title("Choose character card files")
                        .add_filter("PNG character cards", &["png"])
                        .pick_files()
                    {
                        self.set_files(files);
                    }
                }
                PendingDialog::Folder => {
                    if let Some(folder) = rfd::FileDialog::new()
                        .set_title("Choose a folder")
                        .pick_folder()
                    {
                        self.set_files(expand_paths(&[folder]));
                    }
                }
            }
            ui.ctx().request_repaint();
        }

        // Keyboard navigation
        let left = ui.input(|i| i.key_pressed(egui::Key::ArrowLeft));
        let right = ui.input(|i| i.key_pressed(egui::Key::ArrowRight));
        if left {
            self.step(-1);
        }
        if right {
            self.step(1);
        }
    }
}

fn apply_theme(ctx: &egui::Context, viewer: &mut Viewer) {
    // Re-detect the OS theme whenever System mode is active and the
    // window regains focus; cheap enough and covers live changes.
    let dark = viewer.effective_dark();
    if viewer.applied_dark != Some(dark) {
        let visuals = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        ctx.set_visuals(visuals);
        viewer.applied_dark = Some(dark);
    }
}

impl Viewer {
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let count = self.files.len();
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_enabled_ui(count > 1 && self.index > 0, |ui| {
                    if ui.button("<").on_hover_text("Previous card").clicked() {
                        self.step(-1);
                    }
                });
                ui.add_enabled_ui(count > 1 && self.index + 1 < count, |ui| {
                    if ui.button(">").on_hover_text("Next card").clicked() {
                        self.step(1);
                    }
                });
                ui.separator();
                if count == 0 {
                    ui.weak("No cards loaded");
                } else {
                    ui.label(format!("{} / {}", self.index + 1, count));
                    let name = self.file_name();
                    let path = self
                        .card
                        .as_ref()
                        .map(|c| c.path.display().to_string())
                        .unwrap_or_default();
                    ui.strong(name).on_hover_text(path);
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("theme")
                        .selected_text(self.theme.label())
                        .show_ui(ui, |ui| {
                            for theme in Theme::ALL {
                                if ui
                                    .selectable_value(&mut self.theme, theme, theme.label())
                                    .changed()
                                {
                                    self.applied_dark = None;
                                    if theme == Theme::System {
                                        self.system_dark = detect_system_dark();
                                    }
                                    ui.ctx().request_repaint();
                                }
                            }
                        });
                    ui.label("Theme:");
                });
            });
        });
    }

    fn data_panel(&mut self, ui: &mut egui::Ui) {
        let Some(card) = self.card.as_ref() else {
            egui::Panel::right("data").show(ui, |ui| {
                ui.weak("Open files or a folder from the File menu.");
            });
            return;
        };
        let display_chunk = &mut self.display_chunk;
        let scroll_pin = &mut self.scroll_pin;
        #[cfg(test)]
        let last_scroll_id = &mut self.last_scroll_id;
        let force_top = *scroll_pin > 0;
        egui::Panel::right("data")
            .resizable(true)
            .default_size(460.0)
            .min_size(240.0)
            .show(ui, |ui| {
                // Stable id across cards. The scroll reset is done
                // deterministically while the pin is active: purge the
                // ScrollArea's persisted state (offset, kinetic velocity,
                // animation targets), pin the offset to 0, and consume
                // residual wheel smoothing, so a freshly loaded tall card
                // cannot end up displayed halfway down.
                let scroll_id = ui.make_persistent_id(egui::IdSalt::new("card_fields"));
                #[cfg(test)]
                {
                    *last_scroll_id = Some(scroll_id);
                }
                if force_top {
                    ui.ctx()
                        .data_mut(|d| d.remove::<egui::containers::scroll_area::State>(scroll_id));
                }
                let mut scroll = ScrollArea::vertical()
                    .id_salt("card_fields")
                    .auto_shrink([false, false]);
                if force_top {
                    scroll = scroll.vertical_scroll_offset(0.0);
                }
                scroll.show(ui, |ui| {
                    for warning in &card.warnings {
                        warning_block(ui, warning);
                    }
                    if card.payloads.is_empty() {
                        warning_block(ui, "No character card data in this file.");
                        return;
                    }
                    let selected = (*display_chunk).min(card.payloads.len() - 1);
                    if card.payloads.len() > 1 {
                        ui.horizontal(|ui| {
                            ui.strong("Embedded chunk:");
                            egui::ComboBox::from_id_salt("chunk_select")
                                .selected_text(card.payloads[selected].label())
                                .show_ui(ui, |ui| {
                                    for (i, payload) in card.payloads.iter().enumerate() {
                                        if ui
                                            .selectable_label(i == selected, payload.label())
                                            .clicked()
                                            && *display_chunk != i
                                        {
                                            *display_chunk = i;
                                            *scroll_pin = SCROLL_PIN_FRAMES;
                                        }
                                    }
                                });
                        });
                        ui.weak(
                            "Duplicate chunks can hold different data; pick the one to display.",
                        );
                    } else {
                        ui.weak(card.payloads[selected].label());
                    }
                    ui.add_space(4.0);
                    let idx = (*display_chunk).min(card.payloads.len() - 1);
                    let payload = &card.payloads[idx];
                    for field in card::fields(&payload.data, payload.spec) {
                        field_block(ui, &field);
                    }
                    raw_json_block(ui, payload);
                });
                if force_top {
                    // Drop adjustments made during this frame (leftover
                    // momentum, queued scroll-requests) before storing.
                    ui.ctx()
                        .data_mut(|d| d.remove::<egui::containers::scroll_area::State>(scroll_id));
                    // Consume any wheel smoothing still queued for this
                    // frame (the ScrollArea may have already applied part
                    // of it; the state purge above undoes that).
                    ui.ctx().input_mut(|i| i.smooth_scroll_delta = Vec2::ZERO);
                    // Keep pinning for the frame budget so decaying wheel
                    // momentum cannot scroll the new card off the top.
                    *scroll_pin = scroll_pin.saturating_sub(1);
                    ui.ctx().request_repaint();
                }
            });
    }

    fn image_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            let outer = ui.available_rect_before_wrap();
            let texture = self.texture_for(ui);
            let path = self.files.get(self.index).cloned();
            let error = path.as_ref().and_then(|p| self.texture_errors.get(p).cloned());

            match texture {
                Some(tex) => {
                    let avail = outer.size();
                    let tex_size = tex.size_vec2();
                    if avail.x > 0.0 && avail.y > 0.0 {
                        let fit = (avail.x / tex_size.x).min(avail.y / tex_size.y);
                        let disp = tex_size * fit * self.zoom;

                        // Wheel zoom while hovering the image area.
                        let hovered = ui.rect_contains_pointer(outer);
                        if hovered {
                            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                            if scroll != 0.0 {
                                self.zoom = (self.zoom * (1.0 + scroll * 0.0015)).clamp(0.1, 40.0);
                            }
                        }

                        let (rect, response) =
                            ui.allocate_exact_size(avail, Sense::click_and_drag());
                        if response.dragged() {
                            self.pan += response.drag_delta();
                        }
                        if response.double_clicked() {
                            self.zoom = 1.0;
                            self.pan = Vec2::ZERO;
                        }
                        let img_rect = egui::Rect::from_center_size(
                            rect.center() + self.pan,
                            disp.max(Vec2::splat(1.0)),
                        );
                        ui.put(
                            img_rect,
                            egui::Image::from_texture(&tex)
                                .fit_to_exact_size(disp)
                                .sense(Sense::hover()),
                        );
                    }
                }
                None => {
                    if let Some(err) = error {
                        warning_block(ui, &err);
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.heading("zd-ccv");
                            ui.weak("Open character cards via File > Open Files... or File > Open Folder...");
                        });
                    }
                }
            }
        });
    }
}

fn warning_block(ui: &mut egui::Ui, text: &str) {
    let warn = ui.visuals().warn_fg_color;
    egui::Frame::group(ui.style())
        .stroke(egui::Stroke::new(1.0, warn.linear_multiply(0.6)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Warning").strong().color(warn));
                ui.label(text);
            });
        });
    ui.add_space(4.0);
}

fn field_block(ui: &mut egui::Ui, field: &card::Field) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.strong(field.label);
            if matches!(field.value, FieldValue::Empty) {
                ui.weak("(not set)");
            }
        });
        match &field.value {
            FieldValue::Empty => {}
            FieldValue::Text(s) => {
                ui.separator();
                selectable_text(ui, s);
            }
            FieldValue::Dialogue(parts) => {
                ui.separator();
                for (i, part) in parts.iter().enumerate() {
                    if parts.len() > 1 {
                        ui.add_space(2.0);
                        ui.weak(format!("Conversation {}", i + 1));
                    }
                    selectable_text(ui, part);
                }
            }
            FieldValue::Strings(items) => {
                ui.separator();
                for (i, item) in items.iter().enumerate() {
                    if items.len() > 1 {
                        ui.add_space(2.0);
                        ui.weak(format!("#{}", i + 1));
                    }
                    selectable_text(ui, item);
                }
            }
            FieldValue::Lorebook(book) => {
                ui.separator();
                lorebook_blocks(ui, book);
            }
            FieldValue::Json(value) => {
                ui.separator();
                let pretty = serde_json::to_string_pretty(value).unwrap_or_default();
                selectable_text(ui, &pretty);
            }
        }
    });
    ui.add_space(4.0);
}

fn lorebook_blocks(ui: &mut egui::Ui, book: &serde_json::Value) {
    if let Some(name) = book.get("name").and_then(|v| v.as_str()) {
        if !name.is_empty() {
            ui.label(RichText::new(name).italics());
        }
    }
    let Some(entries) = book.get("entries").and_then(|v| v.as_array()) else {
        selectable_text(ui, &serde_json::to_string_pretty(book).unwrap_or_default());
        return;
    };
    ui.weak(format!("{} entries", entries.len()));
    for (i, entry) in entries.iter().enumerate() {
        let comment = entry
            .get("comment")
            .and_then(|v| v.as_str())
            .or_else(|| entry.get("name").and_then(|v| v.as_str()))
            .map(|s| s.to_string());
        let keys = entry
            .get("keys")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let title = match (comment, keys.is_empty()) {
            (Some(c), false) => format!("[{}] {c}", i + 1),
            (Some(c), true) => format!("[{}] {c}", i + 1),
            (None, _) => format!("[{}] {keys}", i + 1),
        };
        egui::CollapsingHeader::new(title).show(ui, |ui| {
            if !keys.is_empty() {
                ui.weak(format!("Keys: {keys}"));
            }
            if let Some(content) = entry.get("content").and_then(|v| v.as_str()) {
                selectable_text(ui, content);
            }
        });
    }
}

fn raw_json_block(ui: &mut egui::Ui, payload: &card::CardPayload) {
    ui.add_space(2.0);
    let title = format!("Raw JSON ({})", payload.kind.keyword());
    egui::CollapsingHeader::new(RichText::new(title).weak()).show(ui, |ui| {
        let pretty = serde_json::to_string_pretty(&payload.raw).unwrap_or_default();
        selectable_text(ui, &pretty);
    });
}

fn selectable_text(ui: &mut egui::Ui, text: &str) {
    ui.add(
        egui::Label::new(RichText::new(text).monospace())
            .selectable(true)
            .wrap_mode(egui::TextWrapMode::Wrap),
    );
}

/// Expand command-line/dialog paths into a sorted list of card files.
/// Folders contribute only their own direct `.png` entries; subfolders
/// are not traversed.
fn expand_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            let entries = match std::fs::read_dir(path) {
                Ok(entries) => entries,
                Err(e) => {
                    eprintln!("could not read folder {}: {e}", path.display());
                    continue;
                }
            };
            for entry in entries.flatten() {
                let child = entry.path();
                if child.is_file() && is_card_file(&child) {
                    files.push(child);
                }
            }
        } else if path.is_file() && is_card_file(path) {
            files.push(path.clone());
        }
    }
    files.sort();
    files.dedup();
    files
}

/// Only PNG files are loaded; anything with another extension is ignored,
/// whether passed explicitly or found in a folder scan.
fn is_card_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("png"))
        .unwrap_or(false)
}

fn main() -> eframe::Result {
    let args = Args::parse();
    let mut files = Vec::new();
    if !args.paths.is_empty() {
        files = expand_paths(&args.paths);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1150.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "zd-ccv",
        options,
        Box::new(move |cc| Ok(Box::new(Viewer::new(cc, files)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use egui::{
        containers::scroll_area::State, Event, Modifiers, MouseWheelUnit, Pos2, RawInput, Rect,
    };

    fn chunk(typ: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(body.len() as u32).to_be_bytes());
        out.extend_from_slice(typ);
        out.extend_from_slice(body);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    fn png_card(name: &str, description: &str) -> Vec<u8> {
        let json = serde_json::json!({
            "spec": "chara_card_v2",
            "spec_version": "2.0",
            "data": {
                "name": name,
                "description": description,
                "personality": "calm",
                "scenario": "",
                "first_mes": "",
                "mes_example": "",
                "creator_notes": "",
                "system_prompt": "",
                "post_history_instructions": "",
                "alternate_greetings": [],
                "tags": [],
                "creator": "",
                "character_version": "",
                "extensions": {}
            }
        });
        let payload = STANDARD.encode(json.to_string());
        let mut body = b"chara".to_vec();
        body.push(0);
        body.extend_from_slice(payload.as_bytes());
        let mut out = vec![0x89u8, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        out.extend_from_slice(&chunk(b"IHDR", &[0u8; 13]));
        out.extend_from_slice(&chunk(b"IDAT", b"fake"));
        out.extend_from_slice(&chunk(b"tEXt", &body));
        out.extend_from_slice(&chunk(b"IEND", b""));
        out
    }

    fn temp_card(tag: &str, name: &str, words: usize) -> PathBuf {
        let description = (0..words)
            .map(|i| format!("alpha{i} beta gamma"))
            .collect::<Vec<_>>()
            .join(" ");
        let mut path = std::env::temp_dir();
        path.push(format!("zd-ccv-scroll-{tag}-{}.png", std::process::id()));
        std::fs::write(&path, png_card(name, &description)).unwrap();
        path
    }

    fn make_viewer(files: Vec<PathBuf>) -> Viewer {
        let first = card::load(&files[0]);
        let mut viewer = Viewer {
            files,
            index: 0,
            card: None,
            textures: HashMap::new(),
            texture_errors: HashMap::new(),
            theme: Theme::Light,
            system_dark: false,
            applied_dark: None,
            zoom: 1.0,
            pan: Vec2::ZERO,
            display_chunk: 0,
            scroll_pin: 0,
            last_scroll_id: None,
            pending_dialog: None,
            startup_dialog_done: true,
        };
        viewer.card = Some(first);
        viewer
    }

    fn frame(ctx: &egui::Context, viewer: &mut Viewer, t: f64, events: Vec<Event>) {
        let raw = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 600.0))),
            time: Some(t),
            events,
            ..Default::default()
        };
        let full_output = ctx.run_ui(raw, |ui| viewer.data_panel(ui));
        // Headless harness: there is no GPU, so drop texture deltas
        // explicitly (font atlas etc.) to satisfy egui's debug assert.
        let mut delta = full_output.textures_delta;
        delta.clear();
        drop(delta);
    }

    fn scroll_offset(ctx: &egui::Context, viewer: &Viewer) -> f32 {
        let id = viewer.last_scroll_id.expect("panel must render");
        State::load(ctx, id)
            .map(|s| s.clamped_offset().y)
            .unwrap_or(0.0)
    }

    fn wheel(in_panel: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(in_panel),
            Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                delta: Vec2::new(0.0, -400.0),
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Wheel,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    #[test]
    fn folder_scan_is_not_recursive() {
        let root = std::env::temp_dir().join(format!("zd-ccv-scan-{}", std::process::id()));
        let sub = root.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(root.join("top.png"), png_card("Top", "d")).unwrap();
        std::fs::write(sub.join("nested.png"), png_card("Nested", "d")).unwrap();
        std::fs::write(root.join("other.webp"), b"RIFFxxxxWEBP").unwrap();

        let files = expand_paths(std::slice::from_ref(&root));
        assert_eq!(
            files,
            vec![root.join("top.png")],
            "only direct .png entries of the folder are loaded"
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn tall_card_switch_shows_panel_at_top() {
        let a = temp_card("a", "Alice", 200);
        let b = temp_card("b", "Bella", 200);
        let ctx = egui::Context::default();
        let mut viewer = make_viewer(vec![a.clone(), b.clone()]);

        for i in 0..3 {
            frame(&ctx, &mut viewer, i as f64 / 60.0, vec![]);
        }
        assert_eq!(scroll_offset(&ctx, &viewer), 0.0, "fresh card at top");

        viewer.step(1);
        for i in 3..12 {
            frame(&ctx, &mut viewer, i as f64 / 60.0, vec![]);
            assert_eq!(
                scroll_offset(&ctx, &viewer),
                0.0,
                "switched to another tall card: panel must show the top (frame {i})"
            );
        }

        std::fs::remove_file(a).ok();
        std::fs::remove_file(b).ok();
    }

    #[test]
    fn switch_pins_top_despite_wheel_momentum() {
        let a = temp_card("c", "Alice", 200);
        let b = temp_card("d", "Bella", 200);
        let ctx = egui::Context::default();
        let mut viewer = make_viewer(vec![a.clone(), b.clone()]);

        let in_panel = Pos2::new(800.0, 300.0);
        // No pin is active in this fixture (viewer built without reload);
        // verify the harness scrolls at all.
        frame(&ctx, &mut viewer, 0.0, vec![Event::PointerMoved(in_panel)]);
        frame(&ctx, &mut viewer, 1.0 / 60.0, wheel(in_panel));
        for i in 2..10 {
            frame(
                &ctx,
                &mut viewer,
                i as f64 / 60.0,
                vec![Event::PointerMoved(in_panel)],
            );
        }
        let scrolled = scroll_offset(&ctx, &viewer);
        assert!(
            scrolled > 100.0,
            "harness sanity: wheel must move the offset, got {scrolled}"
        );

        // Switch cards while the pointer hovers the panel and the wheel
        // smoothing is still feeding the scroll area.
        viewer.step(1);
        frame(
            &ctx,
            &mut viewer,
            10.0 / 60.0,
            vec![Event::PointerMoved(in_panel)],
        );
        for i in 11..30 {
            frame(
                &ctx,
                &mut viewer,
                i as f64 / 60.0,
                vec![Event::PointerMoved(in_panel)],
            );
            assert_eq!(
                scroll_offset(&ctx, &viewer),
                0.0,
                "new card must stay at top despite wheel momentum (frame {i})"
            );
        }
        // After the pin budget expires, the fully decayed momentum must
        // no longer be able to move the panel away from the top.
        for i in 30..60 {
            frame(
                &ctx,
                &mut viewer,
                i as f64 / 60.0,
                vec![Event::PointerMoved(in_panel)],
            );
        }
        assert!(
            scroll_offset(&ctx, &viewer) < 2.0,
            "residual momentum after the pin window must be negligible, got {}",
            scroll_offset(&ctx, &viewer)
        );

        std::fs::remove_file(a).ok();
        std::fs::remove_file(b).ok();
    }
}
