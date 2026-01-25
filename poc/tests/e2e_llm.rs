use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use yt_sl_extractor::contexts::session::domain::commands::StartExtractionSessionCommand;
use yt_sl_extractor::contexts::session::domain::state::{SessionState, SessionStatus, SlideState};
use yt_sl_extractor::contexts::session::infrastructure::orchestrator::SessionOrchestrator;
use yt_sl_extractor::contexts::video::infrastructure::VideoMetadata;
use yt_sl_extractor::shared::domain::{config::LlmConfig, Id, Session};

#[tokio::test]
async fn test_pipeline_with_llm_integration() {
    // 1. Setup Mock LLM Server
    let mock_server = MockServer::start().await;

    // We'll mock two responses:
    // First call -> "SLIDE"
    // Second call -> "NOT_SLIDE"
    // Since requests happen in parallel or indeterminate order, strict ordering might be tricky.
    // However, we can use a custom matcher or simply return "SLIDE" for everything
    // and verify the integration works (i.e., request is made).
    // To test "NOT_SLIDE", let's just use one slide for now to keep it deterministic
    // and prove the integration works.
    // OR: We can use the image content in the request to differentiate, but the base64 string is huge.
    // Let's just verify "SLIDE" case first (Happy Path).

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [
                {
                    "message": {
                        "content": "SLIDE"
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    // 2. Setup File System State
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().to_str().unwrap().to_string();
    let session_id = Id::<Session>::new();
    let session_dir = temp_dir.path().join(session_id.to_string());
    let slides_dir = session_dir.join("slides");

    std::fs::create_dir_all(&slides_dir).expect("Failed to create slides dir");

    // Create a dummy slide image (Valid JPEG)
    let slide_path = slides_dir.join("slide_001.jpg");

    // Create a simple 100x100 white image with some text (conceptually, but here just pixels)
    // We just need it to be a valid JPEG so Tesseract doesn't crash.
    let img = image::ImageBuffer::from_fn(100, 100, |_x, _y| image::Rgb([255u8, 255u8, 255u8]));
    img.save(&slide_path)
        .expect("Failed to save dummy slide image");

    // 3. Create Session State (Skipping download/transcribe/dedup)
    let state_path = session_dir.join("session.json");
    let initial_state = SessionState {
        session_id: session_id.clone(),
        status: SessionStatus::UniqueSlidesIdentified,
        video_metadata: Some(VideoMetadata {
            title: "Test Video".to_string(),
            duration: 60,
            width: 1920,
            height: 1080,
            uploader: "Tester".to_string(),
            upload_date: "20240101".to_string(),
            view_count: Some(100),
            age_limit: 0,
        }),
        video_path: Some("/tmp/fake_video.mp4".to_string()), // Not used in this step
        frames_dir: Some("/tmp/fake_frames".to_string()),    // Not used in this step
        slides_dir: Some(slides_dir.to_str().unwrap().to_string()),
        transcription: None, // Optional
        report_path: None,
        cleaned_report_path: None,
        slides: vec![SlideState {
            slide_index: 1,
            timestamp: 10.0,
            image_path: slide_path.to_str().unwrap().to_string(),
            requires_human_review: false, // Default, LLM might change this if it said NOT_SLIDE
        }],
    };

    let state_json = serde_json::to_string_pretty(&initial_state).unwrap();
    std::fs::write(&state_path, state_json).expect("Failed to write session state");

    // 4. Run Orchestrator
    let command = StartExtractionSessionCommand {
        session_id,
        youtube_url: "https://www.youtube.com/watch?v=g0047beVND4".to_string(), // Dummy URL
        output_dir,
        frame_interval_secs: 5,
        similarity_threshold: 0.95,
        confidence_threshold: 0.6,
        languages: vec!["eng".to_string()],
        llm_config: Some(LlmConfig {
            api_key: Some("fake-key".to_string()),
            api_base: mock_server.uri(), // Point to mock server
            model: "test-model".to_string(),
            prompt: "Is this a slide?".to_string(),
        }),
        generate_pdf: false,
        pdf_template: None,
    };

    // Act
    // We expect this to resume from UniqueSlidesIdentified, run LLM verification, then OCR/Report
    let result = SessionOrchestrator::run_session(command, None).await;

    // Assert
    assert!(result.is_ok(), "Session failed: {:?}", result.err());
    let event = result.unwrap();

    // Verify LLM endpoint was hit exactly once (for the one slide)
    let requests = mock_server
        .received_requests()
        .await
        .expect("No requests received");
    assert_eq!(requests.len(), 1, "Expected 1 call to LLM API");

    // Verify LLM verification happened
    // The "UniqueSlidesIdentified" step is where LLM verification is inserted (step 4b).
    // Wait, let's check the code:
    // // 4b. Optional LLM Verification
    // if state.status == SessionStatus::UniqueSlidesIdentified && command.llm_config.is_some() { ... }

    // Since we set status to UniqueSlidesIdentified, it should enter this block!

    // We returned "SLIDE", so requires_human_review should be false.
    assert_eq!(event.review_count, 0, "Expected 0 slides for review");

    // Also verify that the report was generated (Step 5)
    assert!(
        std::path::Path::new(&event.file_path).exists(),
        "Report should exist"
    );
}
