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

    // Windows (Production): Run GUI
    #[cfg(target_os = "windows")]
    {
        let mut viewport_builder = egui::ViewportBuilder::default()
            .with_decorations(false) // Borderless
            .with_transparent(true)  // Transparent
            .with_always_on_top()
            .with_inner_size([80.0, 120.0]) // Vertical layout
            .with_position(egui::Pos2::new(100.0, 100.0));

        viewport_builder = viewport_builder.with_taskbar(false).with_always_on_top();
        
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

    // macOS/Others (Simulation): TUI Simulation with ANSI codes
    #[cfg(not(target_os = "windows"))]
    {
        use std::io::{Write, stdout};
        use std::thread::sleep;
        use std::time::Duration;

        // Clear screen and hide cursor
        print!("\x1b[2J\x1b[?25l"); 
        
        println!("\x1b[1;1HPopWin Simulation (macOS TUI Mode)");
        println!("\x1b[3;1HWaiting for text selection...");

        loop {
            if let Ok(event) = rx.recv() {
                match event {
                    AppEvent::SelectionDetected { text, position } => {
                        // Draw centered window
                        let x = 10;
                        let y = 5;
                        let width = 25;
                        let text_display = if text.len() > 10 {
                            format!("{}...", &text[..10])
                        } else {
                            format!("{: <13}", text) // Pad to 13 chars (25 - 12 prefix)
                        };

                        // Simulate Fade-In Animation
                        for i in 1..=5 {
                            print!("\x1b[2J"); // Clear
                            print!("\x1b[1;1HPopWin Simulation (macOS TUI Mode)");
                            
                            // Draw fake window with increasing brightness/frame
                            let color = 30 + i; // 31-35 (Red to Magenta etc, simpler than RGB)
                            let frame_color = if i < 3 { 90 } else { 37 }; // Dark gray to White
                            
                            // Top border
                            print!("\x1b[{};{}H\x1b[{}m+-------------------------+", y, x, frame_color);
                            // Title
                            print!("\x1b[{};{}H|  \x1b[1mPopWin Toolbar\x1b[0m\x1b[{}m         |", y+1, x, frame_color);
                            // Separator
                            print!("\x1b[{};{}H|-------------------------|", y+2, x);
                            // Buttons
                            print!("\x1b[{};{}H|  [C] Copy               |", y+3, x);
                            print!("\x1b[{};{}H|  [X] Cut                |", y+4, x);
                            print!("\x1b[{};{}H|  [V] Paste              |", y+5, x);
                            print!("\x1b[{};{}H|                         |", y+6, x);
                            // Selected Text (Fixed width)
                            print!("\x1b[{};{}H|  Selected: \x1b[36m{}\x1b[0m\x1b[{}m|", y+7, x, text_display, frame_color); 
                            // Bottom border
                            print!("\x1b[{};{}H+-------------------------+\x1b[0m", y+8, x);
                            
                            print!("\x1b[{};{}H(Animation Frame: {}/5)", y+13, x, i);
                            stdout().flush().unwrap();
                            sleep(Duration::from_millis(150));
                        }

                        // Interaction simulation
                        sleep(Duration::from_secs(1));
                        print!("\x1b[{};{}H\x1b[32m> User clicked [Copy]\x1b[0m", y+10, x);
                        stdout().flush().unwrap();
                        sleep(Duration::from_secs(1));
                        
                        actions::copy_selection(&text);
                        print!("\x1b[{};{}H\x1b[32m> Copied to clipboard!\x1b[0m", y+11, x);
                        stdout().flush().unwrap();
                        
                        sleep(Duration::from_secs(2));
                        break;
                    }
                    AppEvent::SelectionCleared => {}
                }
            }
        }
        
        // Reset cursor and clear
        print!("\x1b[?25h\nDone.\n");
        Ok(())
    }
}
