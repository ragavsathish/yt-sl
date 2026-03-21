use std::io::{self, Write};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use yt_sl_extractor::cli::{CliArgs, CliProgressReporter};
use yt_sl_extractor::contexts::session::domain::commands::StartExtractionSessionCommand;
use yt_sl_extractor::contexts::session::infrastructure::orchestrator::SessionOrchestrator;
use yt_sl_extractor::shared::domain::Id;
use yt_sl_extractor::shared::infrastructure::logging::{init_logging, LoggingConfig};

#[tokio::main]
async fn main() {
    // 1. Parse CLI Arguments (before logging so we can set log dir)
    let args = CliArgs::from_args();

    // 2. Initialize Logging with file output
    let log_dir = args.output_dir.join("logs");
    let config = LoggingConfig::new().log_directory(log_dir);
    let _guard = match init_logging(config) {
        Ok((subscriber, guard)) => {
            tracing::subscriber::set_global_default(subscriber)
                .expect("Setting default subscriber failed");
            Some(guard)
        }
        Err(e) => {
            eprintln!(
                "Warning: Failed to initialize file logging: {}. Using console only.",
                e
            );
            let subscriber = tracing_subscriber::FmtSubscriber::builder()
                .with_max_level(tracing::Level::INFO)
                .with_target(false)
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("Setting default subscriber failed");
            None
        }
    };

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
    let start_time = Instant::now();

    // 3. Initialize Progress Reporter
    let progress = Arc::new(Mutex::new(CliProgressReporter::new()));

    // 4. Prepare Command
    let session_id = if let Some(ref resume_id) = args.resume_session {
        Id::from_str(resume_id).unwrap_or_else(|_| {
            error!("Invalid session ID: {}", resume_id);
            std::process::exit(1);
        })
    } else {
        Id::new()
    };

    let mut command = StartExtractionSessionCommand {
        session_id,
        youtube_url: args.youtube_url,
        output_dir: args.output_dir.to_string_lossy().to_string(),
        frame_interval_secs: args.interval as u64,
        similarity_threshold: args.threshold,
        confidence_threshold: args.confidence_threshold,
        languages: args.languages,
        llm_config: None,
        generate_pdf: args.generate_pdf,
        pdf_template: args.pdf_template,
        keep_temp: args.keep_temp,
        template: args.template,
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
            let elapsed = start_time.elapsed();

            // Display summary report
            println!("\n========================================");
            println!("  Extraction Summary");
            println!("========================================");
            println!("  Unique slides extracted: {}", event.slide_count);
            println!("  Total frames processed:  {}", event.total_frames);
            println!("  Processing time:         {:.1}s", elapsed.as_secs_f64());
            if let Some(ref stats) = event.ocr_stats {
                println!(
                    "  OCR success rate:        {:.0}%",
                    stats.success_rate * 100.0
                );
                println!("  Avg OCR confidence:      {:.2}", stats.avg_confidence);
                if stats.low_confidence_count > 0 {
                    println!("  Low confidence slides:   {}", stats.low_confidence_count);
                }
            }
            if event.review_count > 0 {
                println!("  Flagged for review:      {}", event.review_count);
            }
            println!("========================================");
            println!("  Markdown: {}", event.file_path);
            if let Some(ref pdf_path) = event.pdf_path {
                println!("  PDF:      {}", pdf_path);
            }
            if let Some(ref cleaned_path) = event.cleaned_file_path {
                println!("  Cleaned:  {}", cleaned_path);
            }
            if let Some(ref cleaned_pdf_path) = event.cleaned_pdf_path {
                println!("  Clean PDF:{}", cleaned_pdf_path);
            }
            println!("========================================\n");

            if event.review_count > 0 {
                print!(
                    "Would you like to delete {} tagged slide images? (y/N): ",
                    event.review_count
                );
                io::stdout().flush().unwrap();

                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
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
                    Err(e) => {
                        warn!(
                            "Failed to read user input for cleanup: {}. Keeping slides.",
                            e
                        );
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
