use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use yt_sl_extractor::cli::{CliArgs, CliProgressReporter};
use yt_sl_extractor::contexts::session::domain::commands::StartExtractionSessionCommand;
use yt_sl_extractor::contexts::session::infrastructure::orchestrator::SessionOrchestrator;
use yt_sl_extractor::shared::domain::Id;

#[tokio::main]
async fn main() {
    // 1. Initialize Logging
    // Use a simpler format when progress bars are active
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    // 2. Parse CLI Arguments
    let args = CliArgs::from_args();
    if let Err(e) = args.validate() {
        error!("Validation error: {}", e);
        std::process::exit(1);
    }

    // 2b. Check External Dependencies
    if let Err(e) = yt_sl_extractor::shared::infrastructure::dependencies::check_dependencies() {
        error!("Dependency check failed: {}", e.user_message());
        std::process::exit(1);
    }

    info!("Starting YouTube Video Slide Extractor...");

    // 3. Initialize Progress Reporter
    let progress = Arc::new(Mutex::new(CliProgressReporter::new()));

    // 4. Prepare Command
    let mut command = StartExtractionSessionCommand {
        session_id: Id::new(),
        youtube_url: args.youtube_url,
        output_dir: args.output_dir.to_string_lossy().to_string(),
        frame_interval_secs: args.interval as u64,
        similarity_threshold: args.threshold,
        confidence_threshold: 0.6, // Default
        languages: args.languages,
        llm_config: None,
    };

    if args.llm_verify {
        command.llm_config = Some(yt_sl_extractor::shared::domain::config::LlmConfig {
            api_key: args.llm_api_key,
            api_base: args.llm_api_base,
            model: args.llm_model,
            prompt: "Analyze this image from a video. Is it a presentation slide (containing text, diagrams, or bullet points) or a view of a person/speaker? Respond with exactly one word: 'SLIDE' or 'NOT_SLIDE'.".to_string(),
        });
    }

    // 5. Run Orchestrator
    info!(
        "Initializing extraction pipeline for session: {}",
        command.session_id
    );
    match SessionOrchestrator::run_session(command, Some(progress)).await {
        Ok(event) => {
            info!("Extraction successful!");
            info!("Report generated at: {}", event.file_path);

            if event.review_count > 0 {
                info!(
                    "Note: {} slides were tagged for manual review (likely speaker views).",
                    event.review_count
                );
                println!();
                print!(
                    "Would you like to delete these {} tagged slide images? (y/N): ",
                    event.review_count
                );
                io::stdout().flush().unwrap();

                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() {
                    let response = input.trim().to_lowercase();
                    if response == "y" || response == "yes" {
                        let mut deleted_count = 0;
                        for path in event.review_slides {
                            if std::fs::remove_file(&path).is_ok() {
                                deleted_count += 1;
                            }
                        }
                        info!(
                            "Successfully deleted {} non-presentation slides.",
                            deleted_count
                        );
                    } else {
                        info!("Kept {} slides for manual review.", event.review_count);
                    }
                }
            }
        }
        Err(e) => {
            error!("Extraction failed: {}", e.user_message());
            std::process::exit(1);
        }
    }
}
