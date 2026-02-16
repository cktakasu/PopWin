#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console in release

use eframe::egui;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;

mod hooks;
mod automation;
mod actions;
mod app;

use app::PopWinApp;

#[derive(Debug, Clone)]
pub enum AppEvent {
    SelectionDetected {
        text: String,
        position: (i32, i32),
    },
    SelectionCleared,
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    info!("Starting PopWin...");

    // Channel for communication between hook thread and UI thread
    let (tx, rx): (Sender<AppEvent>, Receiver<AppEvent>) = unbounded();

    // Start global hook in a background thread
    let tx_clone = tx.clone();
    hooks::start_global_hook(tx_clone);

    // TODO: In a real app, we might need a separate thread for the automation/text extraction logic 
    // to avoid blocking the hook thread, but for this PoC architecture:
    // 1. Hook detects drag -> sends event to Main or Automation Thread?
    // Current design in hooks/mod.rs: 
    // It detects drag, then spawns a thread to get text, then sends AppEvent::SelectionDetected.
    // This is fine for PoC.

    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_decorations(false) // Borderless
        .with_transparent(true)  // Transparent
        .with_always_on_top()
        .with_inner_size([80.0, 120.0]) // Vertical layout: narrow and tall
        .with_position(egui::Pos2::new(100.0, 100.0)); // Initial

    #[cfg(target_os = "windows")]
    {
        viewport_builder = viewport_builder.with_taskbar(false);
    }
    
    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    eframe::run_native(
        "PopWin",
        options,
        Box::new(|cc| Box::new(PopWinApp::new(cc, rx))),
    )
}
