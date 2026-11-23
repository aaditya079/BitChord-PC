use bitchord_core::cipher::solver::CipherSolver;
use bitchord_core::innertube::client::InnertubeClient;

#[tokio::test]
async fn test_cipher_solver_basic() {
    let solver = CipherSolver::new();
    let cipher = "s=abcdef123456&sp=sig&url=https%3A%2F%2Frr1---sn.googlevideo.com%2Fvideoplayback";
    let solved = solver.solve_signature_cipher(cipher, None).await.unwrap();
    assert!(solved.contains("sig=abcdef123456"));
    assert!(solved.contains("https://rr1---sn.googlevideo.com/videoplayback"));
}

#[tokio::test]
async fn test_n_parameter_passthrough_when_no_js() {
    let solver = CipherSolver::new();
    let url = "https://rr1---sn.googlevideo.com/videoplayback?sparams=expire&n=testNCode123";
    let solved = solver.solve_n_parameter(url, None).await.unwrap();
    assert!(solved.contains("n=testNCode123"));
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_innertube_get_stream_network() {
    let client = InnertubeClient::new();
    // Rick Astley - Never Gonna Give You Up (dQw4w9WgXcQ)
    let stream = client.get_stream("dQw4w9WgXcQ").await.unwrap();
    println!(
        "Stream resolved: codec={} bitrate={} sample_rate={} direct={}",
        stream.codec, stream.bitrate, stream.sample_rate, stream.is_direct
    );
    assert!(!stream.url.is_empty());
    assert!(stream.bitrate > 0);
}
