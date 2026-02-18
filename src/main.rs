#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console in release

#[cfg(target_os = "windows")]
use eframe::egui;
#[cfg(not(target_os = "windows"))]
use eframe::egui as _; // Keep it if needed for AppEvent but let's check.
// AppEvent uses deriving Debug, Clone which are standard. 
// It doesn't use egui types. So egui is only for run_native.
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;

mod hooks;
mod automation;
mod actions;
mod app;

#[cfg(target_os = "windows")]
use app::PopWinApp;

#[derive(Debug, Clone)]
pub enum AppEvent {
    SelectionDetected {
        text: String,
        position: (i32, i32),
    },
    SelectionCleared,
    TranslationReceived(String),
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    info!("Starting PopWin...");

    // Channel for communication between hook thread and UI thread
    let (tx, rx): (Sender<AppEvent>, Receiver<AppEvent>) = unbounded();

    // Start global hook in a background thread
    let tx_clone = tx.clone();
    hooks::start_global_hook(tx_clone);

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

        let tx_options = tx.clone();
        eframe::run_native(
            "PopWin",
            options,
            Box::new(|cc| Box::new(PopWinApp::new(cc, rx, tx_options))),
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

        // Keep track of window state for redraw
        let mut last_text = String::new();
        let mut last_pos = (10, 5); // Default sim pos

        loop {
            if let Ok(event) = rx.recv() {
                match event {
                    AppEvent::SelectionDetected { text, position: _ } => {
                        last_text = text.clone();
                        // Draw centered window
                        let x = 10;
                        let y = 5;
                        last_pos = (x, y);

                        let text_display = if text.len() > 10 {
                            format!("{}...", &text[..10])
                        } else {
                            format!("{: <13}", text)
                        };

                        // Simulate Fade-In Animation
                        for i in 1..=5 {
                            print!("\x1b[2J"); // Clear
                            print!("\x1b[1;1HPopWin Simulation (macOS TUI Mode)");
                            
                            let frame_color = if i < 3 { 90 } else { 37 }; // Dark gray to White
                            
                            // Top border
                            print!("\x1b[{};{}H\x1b[{}m+-------------------------+", y, x, frame_color);
                            // Title
                            print!("\x1b[{};{}H|  \x1b[1mPopWin Toolbar\x1b[0m\x1b[{}m         |", y+1, x, frame_color);
                            // Separator
                            print!("\x1b[{};{}H|-------------------------|", y+2, x);
                            // Buttons
                            print!("\x1b[{};{}H|  [C] Copy   [V] Paste    |", y+3, x);
                            print!("\x1b[{};{}H|  [S] Search [E] EN       |", y+4, x);
                            print!("\x1b[{};{}H|                         |", y+5, x);
                            // Selected Text
                            print!("\x1b[{};{}H|  Selected: \x1b[36m{}\x1b[0m\x1b[{}m|", y+6, x, text_display, frame_color); 
                            // Bottom border
                            print!("\x1b[{};{}H+-------------------------+\x1b[0m", y+7, x);
                            
                            print!("\x1b[{};{}H(Animation Frame: {}/5)", y+13, x, i);
                            stdout().flush().unwrap();
                            sleep(Duration::from_millis(150));
                        }

                        // Interaction simulation: Click EN
                        sleep(Duration::from_secs(1));
                        print!("\x1b[{};{}H\x1b[32m> User clicked [EN] (Requesting Translation...)\x1b[0m", y+10, x);
                        stdout().flush().unwrap();
                        
                        // Call async translation
                        actions::translate_async(&text, tx.clone());
                    }
                    AppEvent::TranslationReceived(translation) => {
                         let (x, y) = last_pos;
                         let text_display = if last_text.len() > 10 {
                            format!("{}...", &last_text[..10])
                        } else {
                            format!("{: <13}", last_text)
                        };
                        let frame_color = 37;

                        // Redraw window with translation result
                        print!("\x1b[2J"); // Clear
                        print!("\x1b[1;1HPopWin Simulation (macOS TUI Mode)");
                        
                        // Re-draw UI (simplified)
                        print!("\x1b[{};{}H\x1b[{}m+-------------------------+", y, x, frame_color);
                        print!("\x1b[{};{}H|  \x1b[1mPopWin Toolbar\x1b[0m\x1b[{}m         |", y+1, x, frame_color);
                        print!("\x1b[{};{}H|-------------------------|", y+2, x);
                        print!("\x1b[{};{}H|  [C] Copy   [V] Paste    |", y+3, x);
                        print!("\x1b[{};{}H|  [S] Search [E] EN       |", y+4, x);
                        print!("\x1b[{};{}H|                         |", y+5, x);
                        print!("\x1b[{};{}H|  Selected: \x1b[36m{}\x1b[0m\x1b[{}m|", y+6, x, text_display, frame_color); 
                        // Translation result
                        print!("\x1b[{};{}H|-------------------------|", y+7, x);
                        let safe_translation: String = translation.chars().take(20).collect();
                        print!("\x1b[{};{}H|  \x1b[33m{: <23}\x1b[0m\x1b[{}m", y+8, x, safe_translation, frame_color); // Simplified padding
                        print!("\x1b[{};{}H|", y+8, x+26); 
                        
                        // Bottom border
                        print!("\x1b[{};{}H+-------------------------+\x1b[0m", y+9, x);

                        print!("\x1b[{};{}H\x1b[32m> Translation Received: {}\x1b[0m", y+12, x, translation);
                        stdout().flush().unwrap();
                        
                        sleep(Duration::from_secs(3));
                        break; // End simulation loop
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
