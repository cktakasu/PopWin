use arboard::Clipboard;
use crossbeam_channel::Sender;
use crate::AppEvent;

pub fn copy_selection(text: &str) {
    if let Ok(mut clipboard) = Clipboard::new() {
        let _: Result<(), _> = clipboard.set_text(text);
    }
}

pub fn paste() {
    // Simulate Ctrl+V
    simulate_ctrl_v();
}

pub fn cut() {
    // Simulate Ctrl+X
    simulate_ctrl_x();
}

pub fn search_perplexity(text: &str) {
    let url = format!("https://www.perplexity.ai/search?q={}", urlencoding::encode(text));
    if let Err(e) = webbrowser::open(&url) {
        log::error!("Failed to open browser: {}", e);
    }
}

pub fn translate_async(text: &str, sender: Sender<AppEvent>) {
    let text = text.to_string();
    std::thread::spawn(move || {
        let result = translate_with_google(&text);
        let _ = sender.send(AppEvent::TranslationReceived(result));
    });
}

fn translate_with_google(text: &str) -> String {
    let client = reqwest::blocking::Client::new();
    let url = "https://translate.googleapis.com/translate_a/single";
    let params = [
        ("client", "gtx"),
        ("sl", "auto"),
        ("tl", "ja"),
        ("dt", "t"),
        ("q", text),
    ];

    match client.get(url).query(&params).send() {
        Ok(resp) => {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>() {
                    // Navigate JSON: [[[ "翻訳結果", "Original", ...], ...], ...]
                    if let Some(sentences) = json.as_array().and_then(|a| a.get(0)).and_then(|v| v.as_array()) {
                        let mut result = String::new();
                        for sentence in sentences {
                            if let Some(s) = sentence.as_array().and_then(|a| a.get(0)).and_then(|v| v.as_str()) {
                                result.push_str(s);
                            }
                        }
                        if !result.is_empty() {
                            return result;
                        }
                    }
                }
            }
            "翻訳エラー".to_string()
        }
        Err(e) => {
            log::error!("Translation failed: {}", e);
            "通信エラー".to_string()
        }
    }
}

// Deprecated synchronous dummy translation for reference
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
    
    format!("翻訳(PoC): {} の日本語訳サンプル", text)
}

#[cfg(target_os = "windows")]
mod windows_input {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_V, VK_X, VK_CONTROL,
    };

    pub fn simulate_ctrl_v() {
        unsafe { send_combo(VK_V); }
    }

    pub fn simulate_ctrl_x() {
        unsafe { send_combo(VK_X); }
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

    pub fn simulate_ctrl_x() {
        println!("Action: Cut (Simulated)");
    }
}

#[cfg(target_os = "windows")]
use windows_input::*;

#[cfg(not(target_os = "windows"))]
use dummy_input::*;
