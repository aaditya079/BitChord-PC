use crate::state::{AppState, PlaybackProgress, TrackDto};
use bitchord_core::innertube::models::{LyricLine, Track};
use tauri::State;

#[tauri::command]
pub async fn play_track(state: State<'_, AppState>, track: TrackDto) -> Result<(), String> {
    let t = Track::from(track);
    state.engine.play_track(t).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_playback(state: State<'_, AppState>) -> Result<bool, String> {
    if state.engine.is_playing() {
        state.engine.pause();
        Ok(false)
    } else {
        state.engine.resume();
        Ok(true)
    }
}

#[tauri::command]
pub async fn pause_playback(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.pause();
    Ok(())
}

#[tauri::command]
pub async fn resume_playback(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.resume();
    Ok(())
}

#[tauri::command]
pub async fn stop_playback(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.stop();
    Ok(())
}

#[tauri::command]
pub async fn set_volume(state: State<'_, AppState>, volume: f32) -> Result<(), String> {
    state.engine.set_volume(volume).await;
    Ok(())
}

#[tauri::command]
pub async fn get_playback_state(state: State<'_, AppState>) -> Result<PlaybackProgress, String> {
    Ok(PlaybackProgress {
        position_ms: state.engine.position().as_millis() as u64,
        duration_ms: state.engine.duration().as_millis() as u64,
        is_playing: state.engine.is_playing(),
    })
}

#[tauri::command]
pub async fn search_tracks(state: State<'_, AppState>, query: String) -> Result<Vec<TrackDto>, String> {
    let results = state.innertube.search(&query).await.map_err(|e| e.to_string())?;
    Ok(results.tracks.into_iter().map(TrackDto::from).collect())
}

#[tauri::command]
pub async fn get_lyrics(
    state: State<'_, AppState>,
    artist: String,
    title: String,
) -> Result<Vec<LyricLine>, String> {
    state
        .innertube
        .get_lyrics(&artist, &title)
        .await
        .map_err(|e| e.to_string())
}

// Queue & Transport Commands
#[tauri::command]
pub async fn enqueue_track(state: State<'_, AppState>, track: TrackDto) -> Result<(), String> {
    state.engine.enqueue(Track::from(track)).await;
    Ok(())
}

#[tauri::command]
pub async fn remove_from_queue(state: State<'_, AppState>, index: usize) -> Result<Option<TrackDto>, String> {
    Ok(state.engine.remove_from_queue(index).await.map(TrackDto::from))
}

#[tauri::command]
pub async fn clear_queue(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.clear_queue().await;
    Ok(())
}

#[tauri::command]
pub async fn get_queue(state: State<'_, AppState>) -> Result<Vec<TrackDto>, String> {
    let q = state.engine.get_queue().await;
    Ok(q.into_iter().map(TrackDto::from).collect())
}

#[tauri::command]
pub async fn get_current_track(state: State<'_, AppState>) -> Result<Option<TrackDto>, String> {
    Ok(state.engine.current_track().await.map(TrackDto::from))
}

#[tauri::command]
pub async fn skip_next(state: State<'_, AppState>) -> Result<Option<TrackDto>, String> {
    state
        .engine
        .skip_next()
        .await
        .map(|opt| opt.map(TrackDto::from))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn skip_previous(state: State<'_, AppState>) -> Result<Option<TrackDto>, String> {
    state
        .engine
        .skip_previous()
        .await
        .map(|opt| opt.map(TrackDto::from))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_repeat_mode(state: State<'_, AppState>, mode: u8) -> Result<(), String> {
    state.engine.set_repeat_mode(mode);
    Ok(())
}

#[tauri::command]
pub async fn get_repeat_mode(state: State<'_, AppState>) -> Result<u8, String> {
    Ok(state.engine.get_repeat_mode())
}

#[tauri::command]
pub async fn toggle_shuffle(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.engine.toggle_shuffle())
}

#[tauri::command]
pub async fn is_shuffle(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.engine.is_shuffle())
}
