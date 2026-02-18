#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use std::time::Duration;
    use std::thread;
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoUninitialize};
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern, UIA_TextPatternId,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_C, VK_CONTROL,
    };
    use arboard::Clipboard;

    pub fn get_selected_text() -> Option<String> {
        if let Some(text) = get_text_via_uia() {
            if !text.trim().is_empty() {
                return Some(text);
            }
        }
        get_text_via_clipboard()
    }

    fn get_text_via_uia() -> Option<String> {
        unsafe {
            let com_guard = ComGuard::new()?;
            let automation: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;
            let focused_element: IUIAutomationElement = automation.GetFocusedElement().ok()?;
            let pattern_obj = focused_element.GetCurrentPattern(UIA_TextPatternId).ok()?;
            let text_pattern: IUIAutomationTextPattern = pattern_obj.cast().ok()?;
            let selection = text_pattern.GetSelection().ok()?;
            let length = selection.Length().ok()?;

            if length > 0 {
                let first_range = selection.GetElement(0).ok()?;
                let text = first_range.GetText(-1).ok()?;
                return Some(text.to_string());
            }
        }
        None
    }

    struct ComGuard;

    impl ComGuard {
        fn new() -> Option<Self> {
            unsafe {
                let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
                if hr.is_ok() {
                    Some(ComGuard)
                } else {
                    Some(ComGuard)
                }
            }
        }
    }

    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe {
                windows::Win32::System::Com::CoUninitialize();
            }
        }
    }

    fn get_text_via_clipboard() -> Option<String> {
        simulate_ctrl_c();
        thread::sleep(Duration::from_millis(100));
        if let Ok(mut clipboard) = Clipboard::new() {
            return clipboard.get_text().ok();
        }
        None
    }

    fn simulate_ctrl_c() {
        unsafe {
            let inputs = [
                INPUT { type_: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_CONTROL, ..Default::default() } } },
                INPUT { type_: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_C, ..Default::default() } } },
                INPUT { type_: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_C, dwFlags: KEYEVENTF_KEYUP, ..Default::default() } } },
                INPUT { type_: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_CONTROL, dwFlags: KEYEVENTF_KEYUP, ..Default::default() } } },
            ];
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod dummy_impl {
    pub fn get_selected_text() -> Option<String> {
        // In simulation, hooks sends the text directly, so this might not be called by hooks dummy
        // but if called:
        Some("Simulation Text via Automation".to_string())
    }
}

#[cfg(target_os = "windows")]
pub use windows_impl::*;

#[cfg(not(target_os = "windows"))]
#[allow(unused_imports)]
pub use dummy_impl::*;
