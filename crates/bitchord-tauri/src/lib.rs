pub mod commands;
pub mod state;

use state::{AppState, PlaybackProgress};
use std::time::Duration;
use tauri::{Emitter, Manager};
use tracing::info;

pub fn run() {
    tracing_subscriber::fmt::init();

    let app_state = AppState::new().expect("Failed to initialize BitChord AppState");

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::play_track,
            commands::toggle_playback,
            commands::pause_playback,
            commands::resume_playback,
            commands::stop_playback,
            commands::set_volume,
            commands::get_playback_state,
            commands::search_tracks,
            commands::get_lyrics,
            commands::enqueue_track,
            commands::remove_from_queue,
            commands::clear_queue,
            commands::get_queue,
            commands::get_current_track,
            commands::skip_next,
            commands::skip_previous,
            commands::set_repeat_mode,
            commands::get_repeat_mode,
            commands::toggle_shuffle,
            commands::is_shuffle,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "windows")]
            {
                use window_vibrancy::{apply_acrylic, apply_mica};
                // Try Mica first (Windows 11), fallback to Acrylic (Windows 10)
                if apply_mica(&window, Some(true)).is_err() {
                    let _ = apply_acrylic(&window, Some((18, 18, 18, 125)));
                }
            }

            // Start 100ms playback progress ticker for the frontend UI
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(100));
                loop {
                    interval.tick().await;

                    if let Some(state) = app_handle.try_state::<AppState>() {
                        let is_playing = state.engine.is_playing();
                        let position_ms = state.engine.position().as_millis() as u64;
                        let duration_ms = state.engine.duration().as_millis() as u64;

                        let payload = PlaybackProgress {
                            position_ms,
                            duration_ms,
                            is_playing,
                        };

                        let _ = app_handle.emit("bitchord://progress", payload);
                    }
                }
            });

            info!("BitChord Tauri v2 window initialized with Mica/Acrylic glass styling");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running BitChord desktop application");
}
