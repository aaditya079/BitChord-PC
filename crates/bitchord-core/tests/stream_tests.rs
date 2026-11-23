use bitchord_core::cache::disk::DiskCache;
use bitchord_core::innertube::client::InnertubeClient;
use bitchord_core::stream::buffer::StreamBuffer;
use bitchord_core::stream::downloader::ChunkDownloader;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;

#[test]
fn test_disk_cache_basic_ops() {
    let cache = DiskCache::new().unwrap();
    let video_id = "test_vid_123";
    let test_bytes = b"BITCHORD_TEST_BYTES_48KHZ_OPUS";
    {
        let mut writer = cache.open_writer(video_id).unwrap();
        use std::io::Write;
        writer.write_all(test_bytes).unwrap();
    }

    assert_eq!(cache.cached_bytes(video_id), test_bytes.len() as u64);

    let mut reader = cache.open_reader(video_id).unwrap();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"BITCHORD_TEST_BYTES_48KHZ_OPUS");

    // Test pruning
    cache.prune(100).unwrap();
}

#[tokio::test]
async fn test_stream_buffer_seeking_mock() {
    let cache = DiskCache::new().unwrap();
    let client = reqwest::Client::new();
    let innertube = Arc::new(InnertubeClient::new());
    let dl = Arc::new(ChunkDownloader::new(
        client,
        "mock_vid".to_string(),
        "http://localhost:9999/mock".to_string(),
        Some(1024),
        innertube,
    ));

    let mut buffer = StreamBuffer::new(dl, &cache, "mock_vid", tokio::runtime::Handle::current()).unwrap();

    // Verify seek behavior
    assert_eq!(buffer.seek(SeekFrom::Start(100)).unwrap(), 100);
    assert_eq!(buffer.seek(SeekFrom::Current(50)).unwrap(), 150);
    assert_eq!(buffer.seek(SeekFrom::End(-24)).unwrap(), 1000);
}
