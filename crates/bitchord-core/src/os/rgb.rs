use crate::os::palette::RgbColor;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::debug;

const OPENRGB_HOST: &str = "127.0.0.1:6742";
const ORGB_MAGIC: &[u8; 4] = b"ORGB";
const OPENRGB_CMD_SET_CLIENT_NAME: u32 = 50;
const OPENRGB_CMD_REQUEST_COUNT: u32 = 0;
const OPENRGB_CMD_UPDATE_LEDS: u32 = 1050;

pub struct OpenRgbClient;

impl OpenRgbClient {
    /// Attempts to sync all active OpenRGB controllers to the target color.
    /// Non-blocking: if OpenRGB is not running, fails silently and immediately.
    pub async fn sync_color(color: RgbColor) {
        let stream = match tokio::time::timeout(
            Duration::from_millis(50),
            TcpStream::connect(OPENRGB_HOST),
        )
        .await
        {
            Ok(Ok(s)) => s,
            _ => return, // Server offline or unreachable
        };

        let mut stream = stream;

        // 1. Send client identification
        let client_name = b"BitChord PC\0";
        if Self::send_packet(&mut stream, 0, OPENRGB_CMD_SET_CLIENT_NAME, client_name).await.is_err() {
            return;
        }

        // 2. Query controller count
        if Self::send_packet(&mut stream, 0, OPENRGB_CMD_REQUEST_COUNT, &[]).await.is_err() {
            return;
        }

        let mut header = [0u8; 16];
        if stream.read_exact(&mut header).await.is_err() {
            return;
        }

        let mut count_buf = [0u8; 4];
        if stream.read_exact(&mut count_buf).await.is_err() {
            return;
        }
        let controller_count = u32::from_le_bytes(count_buf);

        // 3. For each controller, set its LEDs to the dominant color
        for dev_idx in 0..controller_count.min(16) {
            // LED data payload: [num_leds: u16] followed by 4-byte colors [r, g, b, 0]
            // We craft a 64-LED broadcast buffer
            let num_leds: u16 = 64;
            let mut payload = Vec::with_capacity(4 + (num_leds as usize) * 4);
            let total_data_len = 2 + (num_leds as u32) * 4;
            payload.extend_from_slice(&total_data_len.to_le_bytes());
            payload.extend_from_slice(&num_leds.to_le_bytes());
            for _ in 0..num_leds {
                payload.push(color.r);
                payload.push(color.g);
                payload.push(color.b);
                payload.push(0); // padding byte
            }

            let _ = Self::send_packet(&mut stream, dev_idx, OPENRGB_CMD_UPDATE_LEDS, &payload).await;
        }

        debug!("OpenRGB ambient lighting synchronized to {:?}", color);
    }

    async fn send_packet(
        stream: &mut TcpStream,
        device_idx: u32,
        command_id: u32,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        let mut packet = Vec::with_capacity(16 + data.len());
        packet.extend_from_slice(ORGB_MAGIC);
        packet.extend_from_slice(&device_idx.to_le_bytes());
        packet.extend_from_slice(&command_id.to_le_bytes());
        packet.extend_from_slice(&(data.len() as u32).to_le_bytes());
        packet.extend_from_slice(data);
        stream.write_all(&packet).await
    }
}
