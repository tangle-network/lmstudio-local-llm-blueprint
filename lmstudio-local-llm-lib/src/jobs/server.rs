use super::*;

/// Tangle Job handler to start the LM Studio API server using `lms server start`.
#[debug_job]
#[instrument(skip(ctx), fields(job_id = START_API_JOB_ID))]
pub async fn start_api(
    Context(ctx): Context<LlmContext>,
    // No arguments needed
    TangleArg(_): TangleArg<()>,
) -> Result<TangleResult<String>> {
    info!("Received start_api job");

    let args = ["server", "start"];
    let output = ctx.run_lms_command(&args).await?;

    info!("Successfully started LM Studio server.");
    Ok(TangleResult(output))
}

/// Tangle Job handler to stop the LM Studio API server using `lms server stop`.
#[debug_job]
#[instrument(skip(ctx), fields(job_id = STOP_API_JOB_ID))]
pub async fn stop_api(
    Context(ctx): Context<LlmContext>,
    // No arguments needed
    TangleArg(_): TangleArg<()>,
) -> Result<TangleResult<String>> {
    info!("Received stop_api job");

    let args = ["server", "stop"];
    let output = ctx.run_lms_command(&args).await?;

    info!("Successfully stopped LM Studio server.");
    Ok(TangleResult(output))
}
