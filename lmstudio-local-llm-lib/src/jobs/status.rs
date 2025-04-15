// src/jobs/status.rs
use super::*;

/// Tangle Job handler to get the status of LM Studio using `lms status`.
#[debug_job]
#[instrument(skip(ctx), fields(job_id = STATUS_JOB_ID))]
pub async fn get_status(
    Context(ctx): Context<LlmContext>,
    // No arguments needed
    TangleArg(_): TangleArg<()>,
) -> Result<TangleResult<String>> {
    info!("Received get_status job");

    let args = ["status"];
    let output = ctx.run_lms_command(&args).await?;

    info!("Successfully retrieved LM Studio status.");
    Ok(TangleResult(output))
}
