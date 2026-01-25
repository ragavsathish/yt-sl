use yt_sl_extractor::contexts::session::domain::commands::StartExtractionSessionCommand;
use yt_sl_extractor::contexts::session::infrastructure::orchestrator::SessionOrchestrator;
use yt_sl_extractor::shared::domain::Id;
use yt_sl_extractor::shared::infrastructure::dependencies::check_dependencies;

#[tokio::test]
async fn test_full_pipeline_with_specific_video() {
    // 0. Check external dependencies
    if let Err(e) = check_dependencies() {
        eprintln!("Skipping E2E test due to missing dependencies: {}", e);
        // We can assert generic failure here if we want strictness,
        // but for a test that might run in restricted envs, return is safer.
        // However, the user ASKED for this test to run. So I should probably let it fail if deps are missing.
        panic!("Missing dependencies for E2E test: {}", e);
    }

    // Setup
    let youtube_url = "https://www.youtube.com/watch?v=g0047beVND4".to_string();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().to_str().unwrap().to_string();

    println!("Running E2E test with output dir: {}", output_dir);

    let command = StartExtractionSessionCommand {
        session_id: Id::new(),
        youtube_url,
        output_dir: output_dir.clone(),
        frame_interval_secs: 10, // Increased interval to speed up test
        similarity_threshold: 0.95,
        confidence_threshold: 0.6,
        languages: vec!["eng".to_string()],
        llm_config: None,
        generate_pdf: true,
        pdf_template: None,
    };

    // Act
    // We pass None for progress reporter as we don't need UI updates in test
    let result = SessionOrchestrator::run_session(command, None).await;

    // Assert
    assert!(result.is_ok(), "Session failed: {:?}", result.err());

    let event = result.unwrap();

    println!("Extraction successful. Slides: {}", event.slide_count);
    println!("Report: {}", event.file_path);

    // Check if files exist
    assert!(
        std::path::Path::new(&event.file_path).exists(),
        "Markdown file should exist"
    );

    if let Some(pdf_path) = event.pdf_path {
        assert!(
            std::path::Path::new(&pdf_path).exists(),
            "PDF file should exist"
        );
    } else {
        println!("Warning: PDF was not generated. Check if Pandoc/Typst is installed.");
    }

    // Verify slide count is reasonable (e.g. > 0)
    assert!(event.slide_count > 0, "Should extract at least one slide");
}
