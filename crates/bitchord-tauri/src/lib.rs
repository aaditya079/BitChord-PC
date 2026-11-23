pub mod commands;
pub mod media_keys;
pub mod state;

use bitchord_core::os::discord::DiscordRpc;
use state::{AppState, PlaybackProgress};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager};
use tracing::info;

pub fn run() {
    tracing_subscriber::fmt::init();

    let app_state = Arc::new(AppState::new().expect("Failed to initialize BitChord AppState"));

    // Start Windows Global Hardware Media Keys Listener
    media_keys::start_media_keys_listener(app_state.clone());

    tauri::Builder::default()
        .manage((*app_state).clone_state())
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
            commands::sync_ambient_rgb,
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

            // Start 100ms playback progress ticker for the frontend UI & Discord RPC
            let app_handle = app.handle().clone();
            let discord_rpc = Arc::new(DiscordRpc::new());

            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(100));
                let mut tick_counter = 0u64;

                loop {
                    interval.tick().await;
                    tick_counter = tick_counter.wrapping_add(1);

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

                        // Broadcast to Discord every 2.5 seconds (every 25 ticks)
                        if tick_counter % 25 == 0 {
                            if let Some(track) = state.engine.current_track().await {
                                let rpc = discord_rpc.clone();
                                let title = track.title;
                                let artist = track.artists.join(", ");
                                let thumb = track.thumbnail_url;

                                tauri::async_runtime::spawn(async move {
                                    rpc.update_presence(
                                        &title,
                                        &artist,
                                        thumb.as_deref(),
                                        is_playing,
                                        position_ms,
                                        duration_ms,
                                    )
                                    .await;
                                });
                            }
                        }
                    }
                }
            });

            info!("BitChord Tauri v2 window initialized with Mica, Media Keys & Discord RPC");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running BitChord desktop application");
}
