//! Dynamic Ambient RGB Backlight Engine
//!
//! Synchronizes keyboard backlights (ASUS TUF / ROG) and gaming peripherals
//! (OpenRGB, SteelSeries GameSense) with the current track's album art.

/// Boosts the extracted album art color's saturation and brightness for punchy LED illumination.
pub fn boost_color(c: egui::Color32) -> (u8, u8, u8) {
    let r = c.r() as f32 / 255.0;
    let g = c.g() as f32 / 255.0;
    let b = c.b() as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let mut h = if delta <= 1e-4 {
        0.0
    } else if (max - r).abs() < 1e-4 {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < 1e-4 {
        60.0 * (((b - r) / delta) + 2.0)
    } else if (max - g).abs() < 1e-4 {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    if h < 0.0 {
        h += 360.0;
    }

    let s = if max <= 1e-4 { 0.0 } else { delta / max };
    // Boost saturation by 35% so LEDs are vivid and not washed out
    let boosted_s = (s * 1.35).clamp(0.4, 1.0);
    // Pin brightness to 1.0 for punchy backlight illumination
    let boosted_v = 1.0f32;

    let c_val = boosted_v * boosted_s;
    let x_val = c_val * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = boosted_v - c_val;

    let (r_prime, g_prime, b_prime) = match (h / 60.0) as i32 {
        0 => (c_val, x_val, 0.0),
        1 => (x_val, c_val, 0.0),
        2 => (0.0, c_val, x_val),
        3 => (0.0, x_val, c_val),
        4 => (x_val, 0.0, c_val),
        _ => (c_val, 0.0, x_val),
    };

    (
        ((r_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// Dispatches the boosted color to all available hardware backends asynchronously.
pub fn sync_ambient_rgb(c: egui::Color32) {
    std::thread::spawn(move || {
        let (r, g, b) = boost_color(c);
        #[cfg(windows)]
        asus_tuf::set_rgb(r, g, b);
        openrgb::set_rgb(r, g, b);
        steelseries::set_rgb(r, g, b);
    });
}

#[cfg(windows)]
mod asus_tuf {
    use windows::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows::Win32::Storage::FileSystem::{CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING};
    use windows::Win32::System::IO::DeviceIoControl;
    use windows::core::w;

    const IOCTL_ASUS_ATK: u32 = 0x0022240C;
    const METHOD_DEVS: u32 = 0x53564544; // 'DEVS' in little-endian ASCII
    const DEV_ID_TUF_KB: u32 = 0x00100056;

    pub fn set_rgb(r: u8, g: u8, b: u8) {
        unsafe {
            let handle = CreateFileW(
                w!(r"\\.\ATKACPI"),
                0x80000000 | 0x40000000, // GENERIC_READ | GENERIC_WRITE
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                None,
            );
            if let Ok(handle) = handle {
                if handle != INVALID_HANDLE_VALUE {
                    // Buffer: [MethodID: 4B] + [Length: 4B (10)] + [DeviceID: 4B] + [Payload: 6B]
                    let mut buf = [0u8; 18];
                    buf[0..4].copy_from_slice(&METHOD_DEVS.to_le_bytes());
                    buf[4..8].copy_from_slice(&10u32.to_le_bytes());
                    buf[8..12].copy_from_slice(&DEV_ID_TUF_KB.to_le_bytes());
                    buf[12] = 0xb4;
                    buf[13] = 0x00; // static mode
                    buf[14] = r;
                    buf[15] = g;
                    buf[16] = b;
                    buf[17] = 0x00; // speed

                    let mut returned: u32 = 0;
                    let _ = DeviceIoControl(
                        handle,
                        IOCTL_ASUS_ATK,
                        Some(buf.as_ptr() as *const _),
                        buf.len() as u32,
                        None,
                        0,
                        Some(&mut returned),
                        None,
                    );
                    let _ = CloseHandle(handle);
                }
            }
        }
    }
}

mod openrgb {
    use std::io::Write;
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    pub fn set_rgb(r: u8, g: u8, b: u8) {
        let Ok(addr) = "127.0.0.1:6742".parse::<SocketAddr>() else { return };
        if let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(40)) {
            let _ = stream.set_write_timeout(Some(Duration::from_millis(40)));
            // OpenRGB protocol packet
            // Magic: 'ORGB', device_idx: 0, command: 1050 (Set Color), length: 4
            let mut packet = Vec::with_capacity(20);
            packet.extend_from_slice(b"ORGB");
            packet.extend_from_slice(&0u32.to_le_bytes());
            packet.extend_from_slice(&1050u32.to_le_bytes());
            packet.extend_from_slice(&4u32.to_le_bytes());
            packet.push(r);
            packet.push(g);
            packet.push(b);
            packet.push(0);
            let _ = stream.write_all(&packet);
        }
    }
}

mod steelseries {
    use std::io::Write;
    use std::net::{SocketAddr, TcpStream};
    use std::path::PathBuf;
    use std::time::Duration;

    pub fn set_rgb(r: u8, g: u8, b: u8) {
        let program_data = std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".into());
        let path = PathBuf::from(program_data).join(r"SteelSeries\SteelSeries Engine 3\coreProps.json");
        let Ok(content) = std::fs::read_to_string(path) else { return };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else { return };
        let Some(raw_addr) = json.get("address").and_then(|a| a.as_str()) else { return };
        let clean_addr = raw_addr.replace("localhost", "127.0.0.1");
        let Ok(addr) = clean_addr.parse::<SocketAddr>() else { return };

        if let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(40)) {
            let _ = stream.set_write_timeout(Some(Duration::from_millis(40)));
            let body = format!(
                r#"{{"game":"BITCHORD","event":"ALBUM_COLOR","data":{{"value":[{},{},{}]}}}}"#,
                r, g, b
            );
            let req = format!(
                "POST /game_event HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                clean_addr,
                body.len(),
                body
            );
            let _ = stream.write_all(req.as_bytes());
        }
    }
}
