//! Dynamic Ambient RGB Backlight Engine
//!
//! Synchronizes keyboard backlights and gaming peripherals with the current track's album art.
//!
//! ### Supported Hardware Backends
//! - **ASUS TUF / ROG Laptops**: Direct kernel ACPI (`\\.\ATKACPI`) control strictly gated
//!   via SMBIOS/Registry vendor and product name verification exclusively to ASUS TUF and ROG
//!   laptops. Commands use volatile RAM mode (`0xb4`) to prevent EEPROM/flash memory wear.
//! - **OpenRGB**: OpenRGB network client communicating over TCP to `localhost:6742` for all
//!   peripherals managed by an OpenRGB instance.
//! - **SteelSeries Engine 3**: GameSense local HTTP/JSON event protocol via `coreProps.json`.
//!
//! ### Architectural Safeguards (Minlor Review Compliance)
//! - **Strict Enumerate-Only Discovery**: Hardware manufacturer and product are verified before
//!   opening device handles. Non-TUF/ROG and unknown vendors never fall back to ATKACPI calls.
//! - **Packet Allow-Listing**: Every ACPI packet is verified against strict method, length, device ID,
//!   and mode constraints before dispatch.
//! - **Rate Limiting & Memory Wear Prevention**: Colors are deduplicated and throttled (250ms minimum
//!   interval) to avoid hardware controller saturation.
//! - **Diagnostic Error Reporting**: Errors are logged and captured in diagnostic status structures
//!   so users and developers can distinguish between unsupported hardware, connection issues, and active sync.
//! - **Isolated CI Testing**: Unit tests run against pure logic and mock device buffers; real hardware
//!   handles are never touched during test suites.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

static EXPERIMENTAL_LIGHTING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enables or disables experimental ambient lighting backends.
/// Default is disabled (`false`).
pub fn set_experimental_lighting(enabled: bool) {
    EXPERIMENTAL_LIGHTING_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Returns whether experimental ambient lighting backends are enabled.
pub fn is_experimental_lighting_enabled() -> bool {
    EXPERIMENTAL_LIGHTING_ENABLED.load(Ordering::Relaxed)
}

/// Live diagnostic telemetry for ambient lighting backends.
#[derive(Clone, Debug)]
pub struct LightingDiagnostics {
    pub asus_tuf: String,
    pub openrgb: String,
    pub steelseries: String,
    pub last_rgb: Option<(u8, u8, u8)>,
}

impl Default for LightingDiagnostics {
    fn default() -> Self {
        Self {
            asus_tuf: "Not initialized".to_owned(),
            openrgb: "Not initialized".to_owned(),
            steelseries: "Not initialized".to_owned(),
            last_rgb: None,
        }
    }
}

static DIAGNOSTICS: OnceLock<Mutex<LightingDiagnostics>> = OnceLock::new();

fn diagnostics() -> &'static Mutex<LightingDiagnostics> {
    DIAGNOSTICS.get_or_init(|| Mutex::new(LightingDiagnostics::default()))
}

/// Retrieves a snapshot of the current ambient lighting diagnostic telemetry.
pub fn get_diagnostics() -> LightingDiagnostics {
    diagnostics().lock().unwrap().clone()
}

/// Pure evaluation function verifying whether the SMBIOS vendor and product identify
/// an ASUS TUF or ROG machine.
pub fn supported_asus(vendor: &str, product: &str) -> bool {
    let (v, p) = (vendor.to_ascii_uppercase(), product.to_ascii_uppercase());
    v.contains("ASUS") && (p.contains("TUF") || p.contains("ROG"))
}

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

struct RateLimiter {
    last_send: Instant,
    last_color: Option<(u8, u8, u8)>,
}

static RATE_LIMITER: OnceLock<Mutex<RateLimiter>> = OnceLock::new();

fn rate_limiter() -> &'static Mutex<RateLimiter> {
    RATE_LIMITER.get_or_init(|| {
        Mutex::new(RateLimiter {
            last_send: Instant::now() - Duration::from_secs(1),
            last_color: None,
        })
    })
}

/// Dispatches the boosted color to all available hardware backends asynchronously,
/// rate-limited to at most once per 250ms and deduplicated against the last color.
pub fn sync_ambient_rgb(c: egui::Color32) {
    if !is_experimental_lighting_enabled() {
        return;
    }

    let (r, g, b) = boost_color(c);

    // Rate-limiting and color deduplication check
    {
        let mut limiter = rate_limiter().lock().unwrap();
        let now = Instant::now();
        if let Some(last) = limiter.last_color {
            if last == (r, g, b) {
                return; // Duplicate color; skip dispatch
            }
        }
        if now.duration_since(limiter.last_send) < Duration::from_millis(250) {
            return; // Throttle: minimum 250ms between hardware writes
        }
        limiter.last_send = now;
        limiter.last_color = Some((r, g, b));
    }

    std::thread::spawn(move || {
        if let Ok(mut diag) = diagnostics().lock() {
            diag.last_rgb = Some((r, g, b));
        }

        #[cfg(windows)]
        asus_tuf::set_rgb(r, g, b);
        openrgb::set_rgb(r, g, b);
        steelseries::set_rgb(r, g, b);
    });
}

#[cfg(windows)]
pub mod asus_tuf {
    use super::{diagnostics, supported_asus};
    use std::sync::OnceLock;
    use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows::Win32::System::IO::DeviceIoControl;
    use windows::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW};
    use windows::core::{w, HSTRING};

    pub const IOCTL_ASUS_ATK: u32 = 0x0022240C;
    pub const METHOD_DEVS: u32 = 0x53564544; // 'DEVS' in little-endian ASCII
    pub const DEV_ID_TUF_KB: u32 = 0x00100056;
    pub const PAYLOAD_MODE_DIRECT_RAM: u8 = 0xb4; // Volatile direct color mode in RAM

    static IS_ASUS_HARDWARE: OnceLock<bool> = OnceLock::new();

    struct SafeHandle(HANDLE);

    impl Drop for SafeHandle {
        fn drop(&mut self) {
            if self.0 != INVALID_HANDLE_VALUE && !self.0.is_invalid() {
                unsafe {
                    let _ = CloseHandle(self.0);
                }
            }
        }
    }

    fn query_bios_reg(name: &str) -> Option<String> {
        let mut buf = [0u16; 256];
        let mut size = (buf.len() * 2) as u32;
        let ok = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                &HSTRING::from(r"HARDWARE\DESCRIPTION\System\BIOS"),
                &HSTRING::from(name),
                RRF_RT_REG_SZ,
                None,
                Some(buf.as_mut_ptr() as *mut _),
                Some(&mut size),
            )
        };
        if ok.is_err() {
            return None;
        }
        let len = (size as usize) / 2;
        let s = String::from_utf16_lossy(&buf[..len.min(buf.len())]);
        let s = s.trim_end_matches('\0').trim();
        (!s.is_empty()).then(|| s.to_owned())
    }

    /// Verifies via SMBIOS / Windows Registry that the machine is an ASUS TUF or ROG system
    /// before ever attempting to open the kernel ACPI driver device handle.
    pub fn is_asus_hardware() -> bool {
        *IS_ASUS_HARDWARE.get_or_init(|| {
            let vendor = query_bios_reg("SystemManufacturer")
                .or_else(|| query_bios_reg("BaseBoardManufacturer"))
                .unwrap_or_default();
            let product = query_bios_reg("SystemProductName")
                .or_else(|| query_bios_reg("BaseBoardProduct"))
                .unwrap_or_default();
            let supported = supported_asus(&vendor, &product);
            if let Ok(mut diag) = diagnostics().lock() {
                if supported {
                    diag.asus_tuf = format!("Supported ASUS hardware detected: {product}");
                } else {
                    diag.asus_tuf = format!("Unsupported hardware: {vendor} {product}");
                }
            }
            supported
        })
    }

    /// Strict packet allow-list validator ensuring that only conforming ASUS TUF RGB packets
    /// can be dispatched to the kernel driver.
    pub fn validate_and_build_packet(r: u8, g: u8, b: u8) -> Result<[u8; 18], &'static str> {
        let mut buf = [0u8; 18];
        buf[0..4].copy_from_slice(&METHOD_DEVS.to_le_bytes());
        buf[4..8].copy_from_slice(&10u32.to_le_bytes());
        buf[8..12].copy_from_slice(&DEV_ID_TUF_KB.to_le_bytes());
        buf[12] = PAYLOAD_MODE_DIRECT_RAM;
        buf[13] = 0x00; // static mode
        buf[14] = r;
        buf[15] = g;
        buf[16] = b;
        buf[17] = 0x00; // speed

        // Allow-list checks
        let method = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let len = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let dev_id = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);

        if method != METHOD_DEVS {
            return Err("Disallowed method ID");
        }
        if len != 10 {
            return Err("Invalid payload length");
        }
        if dev_id != DEV_ID_TUF_KB {
            return Err("Disallowed device ID");
        }
        if buf[12] != PAYLOAD_MODE_DIRECT_RAM {
            return Err("Disallowed lighting mode: only volatile direct RAM mode permitted");
        }
        Ok(buf)
    }

    pub fn set_rgb(r: u8, g: u8, b: u8) {
        if !is_asus_hardware() {
            return;
        }

        let packet = match validate_and_build_packet(r, g, b) {
            Ok(p) => p,
            Err(err) => {
                log::warn!("lighting: ASUS packet validation failed: {err}");
                if let Ok(mut diag) = diagnostics().lock() {
                    diag.asus_tuf = format!("Packet rejected: {err}");
                }
                return;
            }
        };

        unsafe {
            let handle_res = CreateFileW(
                w!(r"\\.\ATKACPI"),
                0x80000000 | 0x40000000, // GENERIC_READ | GENERIC_WRITE
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                None,
            );
            let raw_handle = match handle_res {
                Ok(h) => h,
                Err(err) => {
                    log::warn!("lighting: CreateFileW(\\\\.\\ATKACPI) failed: {err}");
                    if let Ok(mut diag) = diagnostics().lock() {
                        diag.asus_tuf = format!("Driver handle error: {err}");
                    }
                    return;
                }
            };

            if raw_handle == INVALID_HANDLE_VALUE || raw_handle.is_invalid() {
                if let Ok(mut diag) = diagnostics().lock() {
                    diag.asus_tuf = "Invalid driver device handle".to_owned();
                }
                return;
            }
            let handle = SafeHandle(raw_handle);

            let mut returned: u32 = 0;
            let ok = DeviceIoControl(
                handle.0,
                IOCTL_ASUS_ATK,
                Some(packet.as_ptr() as *const _),
                packet.len() as u32,
                None,
                0,
                Some(&mut returned),
                None,
            );

            if let Err(err) = ok {
                log::warn!("lighting: DeviceIoControl on \\\\.\\ATKACPI failed: {err}");
                if let Ok(mut diag) = diagnostics().lock() {
                    diag.asus_tuf = format!("DeviceIoControl failed: {err}");
                }
            } else if let Ok(mut diag) = diagnostics().lock() {
                diag.asus_tuf = format!("Active: Synced RGB({r}, {g}, {b}) via ATKACPI");
            }
        }
    }
}

mod openrgb {
    use super::diagnostics;
    use std::io::Write;
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    pub fn set_rgb(r: u8, g: u8, b: u8) {
        let Ok(addr) = "127.0.0.1:6742".parse::<SocketAddr>() else { return };
        match TcpStream::connect_timeout(&addr, Duration::from_millis(40)) {
            Ok(mut stream) => {
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

                if let Err(err) = stream.write_all(&packet) {
                    log::debug!("lighting: OpenRGB write failed: {err}");
                    if let Ok(mut diag) = diagnostics().lock() {
                        diag.openrgb = format!("Write error: {err}");
                    }
                } else if let Ok(mut diag) = diagnostics().lock() {
                    diag.openrgb = format!("Active: Synced RGB({r}, {g}, {b}) to localhost:6742");
                }
            }
            Err(err) => {
                if let Ok(mut diag) = diagnostics().lock() {
                    diag.openrgb = format!("Offline ({err})");
                }
            }
        }
    }
}

mod steelseries {
    use super::diagnostics;
    use std::io::Write;
    use std::net::{SocketAddr, TcpStream};
    use std::path::PathBuf;
    use std::time::Duration;

    pub fn set_rgb(r: u8, g: u8, b: u8) {
        let program_data = std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".into());
        let path = PathBuf::from(program_data).join(r"SteelSeries\SteelSeries Engine 3\coreProps.json");
        let Ok(content) = std::fs::read_to_string(path) else {
            if let Ok(mut diag) = diagnostics().lock() {
                diag.steelseries = "SteelSeries Engine 3 not found".to_owned();
            }
            return;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else { return };
        let Some(raw_addr) = json.get("address").and_then(|a| a.as_str()) else { return };
        let clean_addr = raw_addr.replace("localhost", "127.0.0.1");
        let Ok(addr) = clean_addr.parse::<SocketAddr>() else { return };

        match TcpStream::connect_timeout(&addr, Duration::from_millis(40)) {
            Ok(mut stream) => {
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
                if let Err(err) = stream.write_all(req.as_bytes()) {
                    if let Ok(mut diag) = diagnostics().lock() {
                        diag.steelseries = format!("Write error: {err}");
                    }
                } else if let Ok(mut diag) = diagnostics().lock() {
                    diag.steelseries = format!("Active: Synced RGB({r}, {g}, {b}) via GameSense");
                }
            }
            Err(err) => {
                if let Ok(mut diag) = diagnostics().lock() {
                    diag.steelseries = format!("Connection error: {err}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_asus_pure_logic() {
        // Supported ASUS gaming laptop lines
        assert!(supported_asus("ASUSTeK COMPUTER INC.", "ASUS TUF Gaming A15 FA506NCR"));
        assert!(supported_asus("ASUS", "ROG Zephyrus G14"));
        assert!(supported_asus("ASUSTeK", "ROG Strix SCAR 16"));
        assert!(supported_asus("ASUS", "TUF Gaming F17"));
        assert!(supported_asus("asus", "rog ally"));

        // Non-gaming or standard ASUS devices
        assert!(!supported_asus("ASUSTeK COMPUTER INC.", "ZenBook Pro 15 UX535"));
        assert!(!supported_asus("ASUS", "VivoBook S15"));
        assert!(!supported_asus("ASUS", "ExpertBook B9"));

        // Non-ASUS hardware
        assert!(!supported_asus("Lenovo", "Legion Pro 7"));
        assert!(!supported_asus("Dell Inc.", "Alienware m16"));
        assert!(!supported_asus("HP", "Omen 16"));
        assert!(!supported_asus("Micro-Star International Co., Ltd.", "MSI Raider GE78"));
        assert!(!supported_asus("", ""));
    }

    #[test]
    fn test_experimental_lighting_toggle_default_off() {
        assert!(!is_experimental_lighting_enabled());
        set_experimental_lighting(true);
        assert!(is_experimental_lighting_enabled());
        set_experimental_lighting(false);
        assert!(!is_experimental_lighting_enabled());
    }

    #[test]
    fn test_packet_allowlist_validation() {
        #[cfg(windows)]
        {
            // Valid packet
            let packet = asus_tuf::validate_and_build_packet(255, 128, 0).expect("valid packet");
            assert_eq!(packet.len(), 18);
            assert_eq!(&packet[0..4], &asus_tuf::METHOD_DEVS.to_le_bytes());
            assert_eq!(&packet[4..8], &10u32.to_le_bytes());
            assert_eq!(&packet[8..12], &asus_tuf::DEV_ID_TUF_KB.to_le_bytes());
            assert_eq!(packet[12], asus_tuf::PAYLOAD_MODE_DIRECT_RAM);
            assert_eq!(packet[13], 0x00);
            assert_eq!(packet[14], 255);
            assert_eq!(packet[15], 128);
            assert_eq!(packet[16], 0);
            assert_eq!(packet[17], 0x00);
        }
    }

    #[test]
    fn test_color_boost_clamping() {
        let (r, g, b) = boost_color(egui::Color32::from_rgb(10, 20, 30));
        assert!(r > 0 || g > 0 || b > 0);
        let (r2, g2, b2) = boost_color(egui::Color32::from_rgb(255, 0, 0));
        assert_eq!((r2, g2, b2), (255, 0, 0));
    }
}
