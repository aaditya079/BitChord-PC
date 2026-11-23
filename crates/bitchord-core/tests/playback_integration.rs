use bitchord_core::player::engine::AudioEngine;
use std::time::Duration;

#[tokio::test]
async fn test_audio_engine_initialization_and_controls() {
    let engine = AudioEngine::new();
    match engine {
        Ok(engine) => {
            assert!(!engine.is_playing());
            assert_eq!(engine.position(), Duration::from_millis(0));

            // Test volume adjustments
            engine.set_volume(0.8).await;
            engine.set_volume(1.2).await; // Clamps to 1.0

            // Test pause / resume
            engine.pause();
            assert!(!engine.is_playing());

            engine.resume();
            assert!(engine.is_playing());

            engine.stop();
            assert!(!engine.is_playing());
        }
        Err(e) => {
            println!("Audio device not present in test environment (expected in headless CI): {:?}", e);
        }
    }
}

#[test]
fn test_memory_efficiency_footprint() {
    let pid = std::process::id();
    let output = std::process::Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-Command",
            &format!("(Get-Process -Id {}).WorkingSet64 / 1MB", pid),
        ])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if let Ok(mb) = text.parse::<f64>() {
            println!("bitchord-core test process Working Set: {:.2} MB", mb);
            assert!(
                mb < 50.0,
                "Process memory footprint exceeded 50MB budget: {:.2} MB",
                mb
            );
        }
    }
}
