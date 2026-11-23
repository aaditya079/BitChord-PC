use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(windows)]
use tokio::net::windows::named_pipe::ClientOptions;
use tracing::debug;

const DISCORD_CLIENT_ID: &str = "1310156948216348692";

pub struct DiscordRpc {
    client_id: String,
}

impl DiscordRpc {
    pub fn new() -> Self {
        Self {
            client_id: DISCORD_CLIENT_ID.to_string(),
        }
    }

    /// Connects to the local Discord IPC named pipe (Windows).
    #[cfg(windows)]
    async fn try_connect() -> Option<tokio::net::windows::named_pipe::NamedPipeClient> {
        for i in 0..10 {
            let pipe_path = format!(r"\\.\pipe\discord-ipc-{}", i);
            if let Ok(client) = ClientOptions::new().open(&pipe_path) {
                return Some(client);
            }
        }
        None
    }

    #[cfg(not(windows))]
    async fn try_connect() -> Option<()> {
        None
    }

    /// Sends a handshaked activity update to Discord without blocking the audio engine.
    pub async fn update_presence(
        &self,
        title: &str,
        artist: &str,
        thumbnail_url: Option<&str>,
        is_playing: bool,
        position_ms: u64,
        duration_ms: u64,
    ) {
        #[cfg(windows)]
        {
            let mut pipe = match Self::try_connect().await {
                Some(p) => p,
                None => {
                    debug!("Discord IPC pipe not available");
                    return;
                }
            };

            // 1. Handshake (Opcode 0)
            let handshake = json!({
                "v": 1,
                "client_id": self.client_id
            })
            .to_string();

            if Self::write_frame(&mut pipe, 0, handshake.as_bytes()).await.is_err() {
                return;
            }

            // Read handshake response
            let mut header = [0u8; 8];
            if pipe.read_exact(&mut header).await.is_err() {
                return;
            }
            let length = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
            let mut buf = vec![0u8; length];
            let _ = pipe.read_exact(&mut buf).await;

            // 2. Set Activity (Opcode 1)
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let (start_time, end_time) = if is_playing && duration_ms > position_ms {
                let start = now - (position_ms / 1000);
                let end = start + (duration_ms / 1000);
                (Some(start), Some(end))
            } else {
                (None, None)
            };

            let payload = json!({
                "cmd": "SET_ACTIVITY",
                "args": {
                    "pid": std::process::id(),
                    "activity": {
                        "details": title,
                        "state": if is_playing { format!("by {}", artist) } else { format!("by {} (Paused)", artist) },
                        "timestamps": {
                            "start": start_time,
                            "end": end_time
                        },
                        "assets": {
                            "large_image": thumbnail_url.unwrap_or("app_icon"),
                            "large_text": "BitChord PC",
                            "small_image": if is_playing { "play" } else { "pause" },
                            "small_text": if is_playing { "Playing" } else { "Paused" }
                        }
                    }
                },
                "nonce": format!("{}", now)
            })
            .to_string();

            let _ = Self::write_frame(&mut pipe, 1, payload.as_bytes()).await;
            debug!("Discord Rich Presence updated for track: {}", title);
        }
    }

    #[cfg(windows)]
    async fn write_frame(
        pipe: &mut tokio::net::windows::named_pipe::NamedPipeClient,
        opcode: u32,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        let mut header = Vec::with_capacity(8 + data.len());
        header.extend_from_slice(&opcode.to_le_bytes());
        header.extend_from_slice(&(data.len() as u32).to_le_bytes());
        header.extend_from_slice(data);
        pipe.write_all(&header).await
    }
}
