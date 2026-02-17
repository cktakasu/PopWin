use eframe::{egui, App, CreationContext, Frame};
use crossbeam_channel::Receiver;

use crate::AppEvent;
use crate::actions;

pub struct PopWinApp {
    visible: bool,
    position: (i32, i32),
    selected_text: String,
    event_receiver: Receiver<AppEvent>,
}

impl PopWinApp {
    pub fn new(_cc: &CreationContext, receiver: Receiver<AppEvent>) -> Self {
        // Customize fonts or style here if needed
        Self {
            visible: false, // Initially hidden
            position: (0, 0),
            selected_text: String::new(),
            event_receiver: receiver,
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

                    // Position toolbar to the left of the selection
                    // Offset left by toolbar width (estimated 80px) and align vertically with selection
                    let x = (position.0 - 90) as f32;
                    let y = position.1 as f32;

                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::Pos2::new(x, y)));
                    ctx.request_repaint();
                }
                AppEvent::SelectionCleared => {
                    self.visible = false;
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

        // Apply custom window styling with animated opacity
        let panel_frame = egui::Frame::window(&ctx.style())
            .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 240))
            .multiply_with_opacity(alpha)
            .rounding(8.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)))
            .inner_margin(8.0);

        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 5.0);

                if ui.button("📋 Copy").clicked() {
                    actions::copy_selection(&self.selected_text);
                    self.visible = false;
                }
                if ui.button("📄 Paste").clicked() {
                    actions::paste();
                    self.visible = false;
                }
            });

            // Optional: Show snippet of text for confirmation
            // let display_text = if self.selected_text.len() > 20 {
            //     format!("{}...", &self.selected_text[..20])
            // } else {
            //     self.selected_text.clone()
            // };
            // ui.weak(display_text);
        });

        // Clicking outside should handle auto-hide - this is tricky with global transparent window
        // For PoC, we rely on "SelectionCleared" event or manual button click.
    }
}
