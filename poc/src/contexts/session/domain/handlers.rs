use crate::contexts::session::domain::commands::StartExtractionSessionCommand;
use crate::contexts::session::domain::events::DocumentGenerated;
use crate::contexts::session::infrastructure::orchestrator::SessionOrchestrator;
use crate::shared::domain::DomainResult;

/// Handles the start extraction session command by delegating to the orchestrator.
pub async fn handle_start_session(
    command: StartExtractionSessionCommand,
) -> DomainResult<DocumentGenerated> {
    SessionOrchestrator::run_session(command).await
}
