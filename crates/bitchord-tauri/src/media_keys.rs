use crate::state::AppState;
use std::sync::Arc;
use tracing::info;
#[cfg(windows)]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY,
};

const HOTKEY_PLAY_PAUSE: i32 = 1001;
const HOTKEY_NEXT_TRACK: i32 = 1002;
const HOTKEY_PREV_TRACK: i32 = 1003;
const HOTKEY_STOP: i32 = 1004;

const VK_MEDIA_NEXT_TRACK: u32 = 0xB0;
const VK_MEDIA_PREV_TRACK: u32 = 0xB1;
const VK_MEDIA_STOP: u32 = 0xB2;
const VK_MEDIA_PLAY_PAUSE: u32 = 0xB3;

/// Spawns a dedicated Windows OS Media Keys worker thread.
/// Captures global hardware media keys (headsets, keyboards, Windows flyout) without lag.
pub fn start_media_keys_listener(state: Arc<AppState>) {
    #[cfg(windows)]
    std::thread::spawn(move || unsafe {
        let null_hwnd = std::ptr::null_mut();

        // Register global hotkeys with NULL hwnd (thread message queue)
        let r1 = RegisterHotKey(null_hwnd, HOTKEY_PLAY_PAUSE, 0, VK_MEDIA_PLAY_PAUSE);
        let r2 = RegisterHotKey(null_hwnd, HOTKEY_NEXT_TRACK, 0, VK_MEDIA_NEXT_TRACK);
        let r3 = RegisterHotKey(null_hwnd, HOTKEY_PREV_TRACK, 0, VK_MEDIA_PREV_TRACK);
        let r4 = RegisterHotKey(null_hwnd, HOTKEY_STOP, 0, VK_MEDIA_STOP);

        info!(
            "Windows global media keys registered (PlayPause={}, Next={}, Prev={}, Stop={})",
            r1 != 0,
            r2 != 0,
            r3 != 0,
            r4 != 0
        );

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_hwnd, 0, 0) > 0 {
            if msg.message == WM_HOTKEY {
                let id = msg.wParam as i32;
                let state_clone = state.clone();

                tauri::async_runtime::spawn(async move {
                    match id {
                        HOTKEY_PLAY_PAUSE => {
                            if state_clone.engine.is_playing() {
                                state_clone.engine.pause();
                            } else {
                                state_clone.engine.resume();
                            }
                        }
                        HOTKEY_NEXT_TRACK => {
                            let _ = state_clone.engine.skip_next().await;
                        }
                        HOTKEY_PREV_TRACK => {
                            let _ = state_clone.engine.skip_previous().await;
                        }
                        HOTKEY_STOP => {
                            state_clone.engine.stop();
                        }
                        _ => {}
                    }
                });
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnregisterHotKey(null_hwnd, HOTKEY_PLAY_PAUSE);
        UnregisterHotKey(null_hwnd, HOTKEY_NEXT_TRACK);
        UnregisterHotKey(null_hwnd, HOTKEY_PREV_TRACK);
        UnregisterHotKey(null_hwnd, HOTKEY_STOP);
    });
}
