use poc::cli::CliArgs;
use poc::contexts::session::domain::commands::StartExtractionSessionCommand;
use poc::contexts::session::infrastructure::orchestrator::SessionOrchestrator;
use poc::shared::domain::Id;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // 1. Initialize Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    info!("Starting YouTube Video Slide Extractor...");

    // 2. Parse CLI Arguments
    let args = CliArgs::from_args();
    if let Err(e) = args.validate() {
        error!("Validation error: {}", e);
        std::process::exit(1);
    }

    // 3. Prepare Command
    let command = StartExtractionSessionCommand {
        session_id: Id::new(),
        youtube_url: args.youtube_url,
        output_dir: args.output_dir.to_string_lossy().to_string(),
        frame_interval_secs: args.interval as u64,
        similarity_threshold: args.threshold,
        confidence_threshold: 0.6, // Default
        languages: args.languages,
    };

    // 4. Run Orchestrator
    info!(
        "Initializing extraction pipeline for session: {}",
        command.session_id
    );
    match SessionOrchestrator::run_session(command).await {
        Ok(event) => {
            info!("Extraction successful!");
            info!("Report generated at: {}", event.file_path);
            info!("Total unique slides: {}", event.slide_count);
        }
        Err(e) => {
            error!("Extraction failed: {}", e.user_message());
            std::process::exit(1);
        }
    }
}
