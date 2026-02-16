use eframe::{egui, App, CreationContext, Frame};
use crossbeam_channel::Receiver;
use std::sync::atomic::{AtomicBool, Ordering};

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
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        // Poll for events from the background thread
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                AppEvent::SelectionDetected { text, position } => {
                    self.selected_text = text;
                    self.position = position;
                    self.visible = true;
                    
                    // Move window to mouse position + offset
                    // Note: eframe window positioning might be limited depending on backend,
                    // but we try to set it via frame if possible, or use Win32 API in main loop.
                    // For now, let's assume frame.set_window_pos works or we handled it main.
                    let x = position.0 as f32;
                    let y = position.1 as f32 + 20.0; // Offset below cursor
                    frame.set_window_pos(egui::Pos2::new(x, y));
                    
                    ctx.request_repaint();
                }
                AppEvent::SelectionCleared => {
                    self.visible = false;
                    ctx.request_repaint();
                }
            }
        }

        if !self.visible {
            frame.set_visible(false);
            // Low CPU usage when hidden: wait for events
            // We sleep a bit to avoid busy loop if event channel is empty? 
            // improved by using request_repaint on event actually.
            return;
        } else {
            frame.set_visible(true);
        }

        // Apply custom window styling for the toolbar look
        let panel_frame = egui::Frame::window(&ctx.style())
            .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 240)) // Dark semi-transparent
            .rounding(8.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)))
            .inner_margin(8.0);

        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(5.0, 0.0);
                
                if ui.button("📋 Copy").clicked() {
                    actions::copy_selection(&self.selected_text);
                    self.visible = false;
                    frame.set_visible(false);
                }
                if ui.button("✂️ Cut").clicked() {
                    actions::cut_selection(); // Note: Cut might need focus handling
                    self.visible = false;
                    frame.set_visible(false);
                }
                if ui.button("📄 Paste").clicked() {
                    actions::paste(); // Note: Paste might need focus handling
                    self.visible = false;
                    frame.set_visible(false);
                }
                if ui.button("🔍 Perplexity").clicked() {
                    actions::search_perplexity(&self.selected_text);
                    self.visible = false;
                    frame.set_visible(false);
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
