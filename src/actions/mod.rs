use arboard::Clipboard;

pub fn copy_selection(text: &str) {
    if let Ok(mut clipboard) = Clipboard::new() {
        let _: Result<(), _> = clipboard.set_text(text);
    }
}

pub fn paste() {
    // Simulate Ctrl+V
    simulate_ctrl_v();
}

pub fn search_perplexity(text: &str) {
    let url = format!("https://www.perplexity.ai/search?q={}", urlencoding::encode(text));
    if let Err(e) = webbrowser::open(&url) {
        log::error!("Failed to open browser: {}", e);
    }
}

pub fn translate(text: &str) -> String {
    let text = text.trim();
    if text.eq_ignore_ascii_case("hello") {
        return "こんにちは (挨拶)".to_string();
    } else if text.eq_ignore_ascii_case("simulation") {
        return "シミュレーション (模擬実験)".to_string();
    } else if text.eq_ignore_ascii_case("popwin") {
        return "ポップウィン (このアプリ)".to_string();
    } else if text.eq_ignore_ascii_case("rust") {
        return "Rust (プログラミング言語)".to_string();
    }
    
    // Default dummy translation for PoC
    format!("翻訳(PoC): {} の日本語訳サンプル", text)
}

#[cfg(target_os = "windows")]
mod windows_input {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_V, VK_CONTROL,
    };

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
    pub fn simulate_ctrl_v() {
        println!("Action: Paste (Simulated)");
    }
}

#[cfg(target_os = "windows")]
use windows_input::*;

#[cfg(not(target_os = "windows"))]
use dummy_input::*;
