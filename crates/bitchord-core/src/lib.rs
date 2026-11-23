//! BITCHORD CORE
//! High-performance streaming audio engine and YouTube Innertube client.

pub mod cipher;
pub mod innertube;
pub mod stream;
pub mod cache;
pub mod player;
pub mod sink;
pub mod os;

pub use innertube::client::InnertubeClient;
pub use player::engine::AudioEngine;
