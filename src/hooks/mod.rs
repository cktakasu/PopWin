use crossbeam_channel::Sender;
use crate::AppEvent;

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};
    use once_cell::sync::Lazy;
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM, HWND};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, MSG, WH_MOUSE_LL,
        WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, MSLLHOOKSTRUCT,
    };

    // Global state for the hook procedure
    static HOOK_SENDER: Lazy<Mutex<Option<Sender<AppEvent>>>> = Lazy::new(|| Mutex::new(None));
    static HOOK_HANDLE: Lazy<Mutex<Option<HHOOK>>> = Lazy::new(|| Mutex::new(None));
    static DRAG_START: Lazy<Mutex<Option<(i32, i32, Instant)>>> = Lazy::new(|| Mutex::new(None));

    const DRAG_THRESHOLD: i32 = 5; // pixels
    const DRAG_TIME_THRESHOLD: u128 = 100; // milliseconds

    pub fn start_global_hook(sender: Sender<AppEvent>) {
        {
            let mut sender_guard = HOOK_SENDER.lock().unwrap();
            *sender_guard = Some(sender);
        }

        thread::spawn(move || {
            unsafe {
                let hook_id = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), None, 0);
                match hook_id {
                    Ok(hhook) => {
                        {
                            let mut handle_guard = HOOK_HANDLE.lock().unwrap();
                            *handle_guard = Some(hhook);
                        }
                        let mut msg = MSG::default();
                        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {}
                    }
                    Err(e) => log::error!("Failed to set WH_MOUSE_LL hook: {:?}", e),
                }
            }
        });
    }

    pub fn stop_global_hook() {
        unsafe {
            let mut handle_guard = HOOK_HANDLE.lock().unwrap();
            if let Some(hhook) = *handle_guard {
                UnhookWindowsHookEx(hhook);
                *handle_guard = None;
            }
        }
    }

    unsafe extern "system" fn hook_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        if n_code >= 0 {
            let mouse_struct = *(l_param.0 as *const MSLLHOOKSTRUCT);
            let x = mouse_struct.pt.x;
            let y = mouse_struct.pt.y;

            match w_param.0 as u32 {
                WM_LBUTTONDOWN => {
                    let mut start_guard = DRAG_START.lock().unwrap();
                    *start_guard = Some((x, y, Instant::now()));
                     if let Some(sender_guard) = HOOK_SENDER.lock().unwrap().as_ref() {
                        let _ = sender_guard.send(AppEvent::SelectionCleared);
                    }
                }
                WM_MOUSEMOVE => {}
                WM_LBUTTONUP => {
                     let mut start_guard = DRAG_START.lock().unwrap();
                     if let Some((start_x, start_y, start_time)) = *start_guard {
                         let distance = ((x - start_x).pow(2) + (y - start_y).pow(2)) as f64;
                         let distance = distance.sqrt() as i32;
                         let elapsed = start_time.elapsed().as_millis();

                         if distance > DRAG_THRESHOLD && elapsed > DRAG_TIME_THRESHOLD {
                             if let Some(sender_guard) = HOOK_SENDER.lock().unwrap().as_ref() {
                                let sender_clone = sender_guard.clone();
                                thread::spawn(move || {
                                    thread::sleep(Duration::from_millis(50)); 
                                    if let Some(text) = crate::automation::get_selected_text() {
                                        if !text.trim().is_empty() {
                                            let _ = sender_clone.send(AppEvent::SelectionDetected {
                                                text,
                                                position: (x, y),
                                            });
                                        }
                                    }
                                });
                             }
                         }
                     }
                     *start_guard = None;
                }
                _ => {}
            }
        }
        CallNextHookEx(None, n_code, w_param, l_param)
    }
}

#[cfg(not(target_os = "windows"))]
mod dummy_impl {
    use super::*;
    use std::thread;
    use std::time::Duration;

    pub fn start_global_hook(sender: Sender<AppEvent>) {
        println!("Starting dummy hook for simulation...");
        thread::spawn(move || {
            // Simulate a selection event after 3 seconds
            thread::sleep(Duration::from_secs(3));
            let _ = sender.send(AppEvent::SelectionDetected {
                text: "Simulation Text".to_string(),
                position: (200, 200),
            });
            println!("Simulated selection event sent!");
        });
    }

    pub fn stop_global_hook() {}
}

#[cfg(target_os = "windows")]
pub use windows_impl::*;

#[cfg(not(target_os = "windows"))]
pub use dummy_impl::*;
