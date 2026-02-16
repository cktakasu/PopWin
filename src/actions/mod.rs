use arboard::Clipboard;
use std::process::Command;
use url::form_urlencoded;

pub fn copy_selection(text: &str) {
    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text(text);
    }
}

pub fn cut_selection() {
    // Cut logic is tricky system-wide without modifying source app directly.
    // For PoC: Simulate Ctrl+X
    simulate_ctrl_x();
}

pub fn paste() {
    // Simulate Ctrl+V
    simulate_ctrl_v();
}

pub fn search_perplexity(text: &str) {
    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .append_pair("q", text)
        .finish();
    let url = format!("https://www.perplexity.ai/search?{}", encoded);
    
    // Open URL in default browser
    if cfg!(target_os = "windows") {
        let _ = Command::new("cmd")
            .args(&["/C", "start", &url])
            .spawn();
    } else if cfg!(target_os = "macos") {
        let _ = Command::new("open")
            .arg(&url)
            .spawn();
    }
}

#[cfg(target_os = "windows")]
mod windows_input {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_C, VK_V, VK_X, VK_CONTROL,
    };

    pub fn simulate_ctrl_x() {
        unsafe { send_combo(VK_X); }
    }

    pub fn simulate_ctrl_v() {
        unsafe { send_combo(VK_V); }
    }

    unsafe fn send_combo(key: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY) {
        let inputs = [
            INPUT { type_: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_CONTROL, ..Default::default() } } },
            INPUT { type_: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: key, ..Default::default() } } },
            INPUT { type_: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: key, dwFlags: KEYEVENTF_KEYUP, ..Default::default() } } },
            INPUT { type_: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_CONTROL, dwFlags: KEYEVENTF_KEYUP, ..Default::default() } } },
        ];
        let _ = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(not(target_os = "windows"))]
mod dummy_input {
    pub fn simulate_ctrl_x() {
        println!("Action: Cut (Simulated)");
    }
    pub fn simulate_ctrl_v() {
        println!("Action: Paste (Simulated)");
    }
}

#[cfg(target_os = "windows")]
use windows_input::*;

#[cfg(not(target_os = "windows"))]
use dummy_input::*;

pub fn cut_selection() {
    simulate_ctrl_x();
}

pub fn paste() {
    simulate_ctrl_v();
}
