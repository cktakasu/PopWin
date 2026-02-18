use eframe::{egui, App, CreationContext, Frame};
use crossbeam_channel::{Receiver, Sender};

use crate::AppEvent;
use crate::actions;

pub struct PopWinApp {
    visible: bool,
    position: (i32, i32),
    selected_text: String,
    translation: Option<String>,
    event_receiver: Receiver<AppEvent>,
    event_sender: Sender<AppEvent>,
}

impl PopWinApp {
    pub fn new(_cc: &CreationContext, receiver: Receiver<AppEvent>, sender: Sender<AppEvent>) -> Self {
        // Customize fonts or style here if needed
        Self {
            visible: false, // Initially hidden
            position: (0, 0),
            selected_text: String::new(),
            translation: None,
            event_receiver: receiver,
            event_sender: sender,
        }
    }
}

impl App for PopWinApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Poll for events from the background thread
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                AppEvent::SelectionDetected { text, position } => {
                    self.selected_text = text;
                    self.position = position;
                    self.visible = true;
                    self.translation = None; // Reset translation
                    ctx.request_repaint();
                }
                AppEvent::SelectionCleared => {
                    self.visible = false;
                    self.translation = None;
                    ctx.request_repaint();
                }
                AppEvent::TranslationReceived(text) => {
                    self.translation = Some(text);
                    ctx.request_repaint();
                }
            }
        }

        // Fade-in/fade-out animation (150ms)
        let alpha = ctx.animate_bool_with_time(
            egui::Id::new("toolbar_fade"),
            self.visible,
            0.15,
        );

        if alpha == 0.0 {
            return;
        }

        // Slide up from bottom: offset decreases from 10px to 0px as alpha goes 0→1
        let slide_offset = (1.0 - alpha) * 10.0;
        let x = (self.position.0 - 90) as f32;
        let y = self.position.1 as f32 + slide_offset;
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::Pos2::new(x, y)));

        // Apply custom window styling with animated opacity
        let panel_frame = egui::Frame::window(&ctx.style())
            .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 240))
            .multiply_with_opacity(alpha)
            .rounding(8.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)))
            .inner_margin(8.0);

        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(5.0, 5.0);

                // Row 1: Clipboard Actions
                ui.horizontal(|ui| {
                    if ui.button("📋 Copy").clicked() {
                        actions::copy_selection(&self.selected_text);
                        self.visible = false;
                    }
                    if ui.button("✂️ Cut").clicked() {
                         // Note: Cut implementation missing in actions for now
                    }
                     if ui.button("📄 Paste").clicked() {
                        actions::paste();
                        self.visible = false;
                    }
                });

                // Row 2: Search & Translate
                ui.horizontal(|ui| {
                    if ui.button("🔍 Perplexity").clicked() {
                        actions::search_perplexity(&self.selected_text);
                        self.visible = false;
                    }
                    if ui.button("A文 EN").clicked() {
                        self.translation = Some("翻訳中...".to_string());
                        actions::translate_async(&self.selected_text, self.event_sender.clone());
                    }
                });

                // Translation Result Area
                if let Some(text) = &self.translation {
                    ui.separator();
                    // Wrap text if too long
                    ui.add(egui::Label::new(egui::RichText::new(text).color(egui::Color32::LIGHT_BLUE)).wrap(true));
                }
            });
        });

        // Clicking outside should handle auto-hide - this is tricky with global transparent window
        // For PoC, we rely on "SelectionCleared" event or manual button click.
    }
}
